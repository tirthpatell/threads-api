//! Recovery from `/threads_publish` false failures.
//!
//! Meta's `/threads_publish` endpoint sometimes returns an error response
//! (HTTP 5xx with error code 10, "Application does not have permission for
//! this action") even though the container was actually published. Per the
//! Threads API troubleshooting docs (section "Publishing Does Not Return a
//! Media ID"), the canonical recovery strategy is to check the container's
//! status and, if it flipped to PUBLISHED, locate the resulting post among
//! the user's recent posts.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::client::Client;
use crate::error;
use crate::http::is_non_retryable_permanent_error_code;
use crate::types::{
    CarouselPostContent, ContainerId, ImagePostContent, MediaType, Post, PostId, PostsOptions,
    TextPostContent, UserId, VideoPostContent,
};

/// Negative buffer applied to the publish-start timestamp when querying
/// recent posts during recovery. It absorbs clock skew between the local
/// machine and Meta's servers without widening the search enough to risk
/// collisions with prior unrelated publishes.
const PUBLISH_RECOVERY_WINDOW: Duration = Duration::from_secs(5);

/// Recovery polling bounds. Both gates (container status flip to PUBLISHED
/// and the `/me/threads` index seeing our new post) can race with Meta
/// returning the code-10 error response, so do a few brief polls before
/// concluding the publish really failed. Total worst-case latency is
/// `(MAX_RECOVERY_STATUS_POLLS + MAX_RECOVERY_LIST_POLLS - 2) *
/// RECOVERY_POLL_INTERVAL`, kept small enough to not dominate caller-visible
/// latency.
const MAX_RECOVERY_STATUS_POLLS: u32 = 5;
const MAX_RECOVERY_LIST_POLLS: u32 = 3;
const RECOVERY_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Returns `true` if `err` matches the documented Meta pattern where
/// `/threads_publish` returns an error response despite the container
/// actually being published. Currently this is code 10 (GraphMethodException),
/// which Meta returns from `/threads_publish` for some app/permission
/// configurations after-the-fact, with the container moving to PUBLISHED
/// regardless.
fn should_attempt_publish_recovery(err: &error::Error) -> bool {
    error::extract_base_fields(err)
        .is_some_and(|fields| is_non_retryable_permanent_error_code(fields.code))
}

impl Client {
    /// Canonical post-publish-error glue for `create_*_post`: attempt
    /// recovery and return the recovered post, or surface the original
    /// publish error when recovery isn't possible.
    ///
    /// The matcher MUST be content-specific enough to be unique per request
    /// even under concurrent publishes from the same user. See each
    /// `*_matcher` helper for the per-type guarantees.
    pub(crate) async fn recover_or_original(
        &self,
        container_id: &str,
        publish_start: DateTime<Utc>,
        publish_err: error::Error,
        matches: impl Fn(&Post) -> bool,
    ) -> crate::Result<Post> {
        // Short-circuit for genuine failures (not code 10), avoiding an
        // extra Meta round-trip.
        if !should_attempt_publish_recovery(&publish_err) {
            return Err(publish_err);
        }

        match self
            .try_recover_published_post(container_id, publish_start, matches)
            .await
        {
            Some(post) => {
                tracing::info!(
                    container_id,
                    post_id = %post.id,
                    "Recovered post after /threads_publish false failure"
                );
                Ok(post)
            }
            // Recovery couldn't help — surface the ORIGINAL publish error;
            // that's what the caller needs to see.
            None => Err(publish_err),
        }
    }

    /// Implements the Meta-documented recovery flow for `/threads_publish`
    /// calls that error out despite succeeding server-side:
    ///
    /// 1. Poll the container's status briefly until it flips to PUBLISHED.
    ///    The publish response can arrive before the container's status row
    ///    has been updated, so a single read can spuriously see FINISHED and
    ///    mistake a successful publish for a real failure.
    /// 2. List recent posts authored by this user since `publish_start`
    ///    (minus a small skew buffer) and apply the caller-supplied matcher.
    ///    The post can lag the container flip in the `/me/threads` index, so
    ///    retry the list lookup briefly before giving up.
    /// 3. Return the unique matching post, or `None` if zero or more than
    ///    one post matches across all attempts.
    async fn try_recover_published_post(
        &self,
        container_id: &str,
        publish_start: DateTime<Utc>,
        matches: impl Fn(&Post) -> bool,
    ) -> Option<Post> {
        // Gate 1: poll until the container reports PUBLISHED. Terminal
        // failure states (ERROR, EXPIRED) short-circuit to "not recovered" —
        // the publish really didn't succeed.
        if !self
            .wait_for_container_published(&ContainerId::from(container_id))
            .await
        {
            return None;
        }

        // Gate 2: poll the user's posts for a matching post. The container
        // just flipped to PUBLISHED but indexing may lag slightly; retry
        // briefly.
        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return None;
        }
        let since_ts = (publish_start
            - chrono::Duration::from_std(PUBLISH_RECOVERY_WINDOW)
                .unwrap_or(chrono::Duration::seconds(5)))
        .timestamp();

        let opts = PostsOptions {
            limit: Some(25),
            since: Some(since_ts),
            before: None,
            after: None,
            until: None,
        };

        for attempt in 0..MAX_RECOVERY_LIST_POLLS {
            if attempt > 0 {
                tokio::time::sleep(RECOVERY_POLL_INTERVAL).await;
            }

            let posts = match self
                .get_user_posts(&UserId::from(user_id.as_str()), Some(&opts))
                .await
            {
                Ok(posts) => posts,
                Err(err) => {
                    tracing::debug!(error = %err, "Recovery: failed to list recent posts");
                    return None;
                }
            };

            // Find a unique match. Fail closed on multiple matches —
            // matchers are designed to be unique per request, so >1 match
            // means we'd be guessing; better to surface the original
            // publish error.
            let mut found: Option<&Post> = None;
            for post in &posts.data {
                if matches(post) {
                    if found.is_some() {
                        return None; // ambiguous
                    }
                    found = Some(post);
                }
            }
            if let Some(post) = found {
                return Some(post.clone());
            }
            // Zero matches yet — fall through to next poll.
        }

        None
    }

    /// Polls the container's status endpoint until it reports PUBLISHED, a
    /// terminal failure state is observed, or the poll budget is exhausted.
    /// Returns `true` only when PUBLISHED is observed.
    async fn wait_for_container_published(&self, container_id: &ContainerId) -> bool {
        for attempt in 0..MAX_RECOVERY_STATUS_POLLS {
            if attempt > 0 {
                tokio::time::sleep(RECOVERY_POLL_INTERVAL).await;
            }

            let status = match self.get_container_status(container_id).await {
                Ok(status) => status,
                Err(err) => {
                    tracing::debug!(error = %err, "Recovery: failed to fetch container status");
                    return false;
                }
            };

            match status.status.as_str() {
                crate::constants::CONTAINER_STATUS_PUBLISHED => return true,
                // Terminal — the publish really did fail. No point polling.
                crate::constants::CONTAINER_STATUS_ERROR
                | crate::constants::CONTAINER_STATUS_EXPIRED => return false,
                // FINISHED or IN_PROGRESS — keep polling.
                _ => {}
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Matchers
// ---------------------------------------------------------------------------

/// Returns `true` if a retrieved post's quote state aligns with what the
/// caller asked for. `None` means "must not be a quote post"; `Some(id)`
/// means "must be a quote of that specific post". Matching across the
/// quote/non-quote boundary would let a regular post masquerade as a quote
/// post (or vice-versa) during recovery.
fn quote_matches(post: &Post, want_quoted_id: Option<&PostId>) -> bool {
    match want_quoted_id {
        None => !post.is_quote_post,
        Some(want) => {
            post.is_quote_post
                && post
                    .quoted_post
                    .as_ref()
                    .is_some_and(|quoted| &quoted.id == want)
        }
    }
}

/// Returns `true` if the (text, quoted_post_id) pair is strong enough to
/// single out one reply among potentially many to the same parent. Without
/// one of these signals we'd be guessing.
fn reply_has_unique_discriminator(text: &str, quoted_post_id: Option<&PostId>) -> bool {
    !text.is_empty() || quoted_post_id.is_some()
}

/// Returns `true` if a non-reply post has any content signal that
/// distinguishes it from prior unrelated posts in the recovery window. An
/// image-only or video-only root with empty text and empty topic_tag has no
/// such signal — matching by media_type + !is_reply alone would accept any
/// prior post of the same type. Such recoveries fail closed.
fn root_has_unique_discriminator(
    text: &str,
    topic_tag: &str,
    quoted_post_id: Option<&PostId>,
) -> bool {
    !text.is_empty() || !topic_tag.is_empty() || quoted_post_id.is_some()
}

/// Returns the parent post ID for a reply from the embedded `replied_to`
/// object, or `None` if it isn't populated.
fn replied_to_id(post: &Post) -> Option<&PostId> {
    post.replied_to.as_ref().map(|parent| &parent.id)
}

fn post_text(post: &Post) -> &str {
    post.text.as_deref().unwrap_or("")
}

fn post_topic_tag(post: &Post) -> &str {
    post.topic_tag.as_deref().unwrap_or("")
}

/// Shared reply/root matching logic for media posts (image, video,
/// carousel). `text` and `topic_tag` are the values the caller asked to
/// publish (empty string when unset).
fn media_content_matches(
    post: &Post,
    text: &str,
    topic_tag: &str,
    reply_to: Option<&PostId>,
    quoted_post_id: Option<&PostId>,
) -> bool {
    if !quote_matches(post, quoted_post_id) {
        return false;
    }
    match reply_to {
        None => {
            if !root_has_unique_discriminator(text, topic_tag, quoted_post_id) {
                return false;
            }
            !post.is_reply && post_text(post) == text && post_topic_tag(post) == topic_tag
        }
        Some(parent) => {
            if !post.is_reply || replied_to_id(post) != Some(parent) {
                return false;
            }
            if !reply_has_unique_discriminator(text, quoted_post_id) {
                return false;
            }
            post_text(post) == text
        }
    }
}

/// Matcher for a text-only post. Text posts always have non-empty text
/// (validated upstream), so text equality is itself a strong discriminator.
/// Quote state must match in both directions, and non-replies additionally
/// compare topic_tag to disambiguate same-text posts across different topics.
pub(crate) fn text_matcher(content: &TextPostContent) -> impl Fn(&Post) -> bool {
    let text = content.text.clone();
    let topic_tag = content.topic_tag.clone().unwrap_or_default();
    let reply_to = content.reply_to_id.clone();
    let quoted = content.quoted_post_id.clone();
    move |post: &Post| {
        // Some text post variants come back as TEXT_POST or omit media_type
        // entirely. Accept both shapes.
        if !matches!(
            post.media_type,
            None | Some(MediaType::TextPost) | Some(MediaType::Text)
        ) {
            return false;
        }
        if !quote_matches(post, quoted.as_ref()) {
            return false;
        }
        if post_text(post) != text {
            return false;
        }
        match reply_to.as_ref() {
            Some(parent) => post.is_reply && replied_to_id(post) == Some(parent),
            None => !post.is_reply && post_topic_tag(post) == topic_tag,
        }
    }
}

/// Matcher for a single-image post.
///
/// For non-replies, exact text + topic_tag + non-reply state is the match
/// key; we additionally require at least one of (text, topic_tag,
/// quoted_post_id) to be non-empty, because an image-only root with no text
/// and no tag has no unique signal.
///
/// For replies, parent ID is necessary but not sufficient (multiple replies
/// to the same parent are valid). Require non-empty text or a matching
/// quoted-post target; blank-text non-quote replies fail closed.
///
/// The image URL stored on the post is Meta's CDN URL, not ours, so we
/// don't compare it.
pub(crate) fn image_matcher(content: &ImagePostContent) -> impl Fn(&Post) -> bool {
    let text = content.text.clone().unwrap_or_default();
    let topic_tag = content.topic_tag.clone().unwrap_or_default();
    let reply_to = content.reply_to_id.clone();
    let quoted = content.quoted_post_id.clone();
    move |post: &Post| {
        if post.media_type != Some(MediaType::Image) {
            return false;
        }
        media_content_matches(post, &text, &topic_tag, reply_to.as_ref(), quoted.as_ref())
    }
}

/// Matcher for a video post; mirrors [`image_matcher`]. After publish, Meta
/// may report video posts as media_type `VIDEO` or `AUDIO` (for audio-only
/// uploads). Accept either.
pub(crate) fn video_matcher(content: &VideoPostContent) -> impl Fn(&Post) -> bool {
    let text = content.text.clone().unwrap_or_default();
    let topic_tag = content.topic_tag.clone().unwrap_or_default();
    let reply_to = content.reply_to_id.clone();
    let quoted = content.quoted_post_id.clone();
    move |post: &Post| {
        if !matches!(
            post.media_type,
            Some(MediaType::Video) | Some(MediaType::Audio)
        ) {
            return false;
        }
        media_content_matches(post, &text, &topic_tag, reply_to.as_ref(), quoted.as_ref())
    }
}

/// Matcher for a published carousel post.
///
/// Note on why we don't match by child IDs: Meta's create endpoint takes
/// MEDIA CONTAINER IDs in the `children` parameter, but the read endpoint
/// (`children` field) returns CHILD POST IDs of the individual published
/// children — these are different IDs. So matching by ID-set never works
/// after publish. Instead match by content: text + topic_tag + reply/quote
/// state + children count, with the same uniqueness rules as image posts.
pub(crate) fn carousel_matcher(content: &CarouselPostContent) -> impl Fn(&Post) -> bool {
    let text = content.text.clone().unwrap_or_default();
    let topic_tag = content.topic_tag.clone().unwrap_or_default();
    let reply_to = content.reply_to_id.clone();
    let quoted = content.quoted_post_id.clone();
    let want_children = content.children.len();
    move |post: &Post| {
        if post.media_type != Some(MediaType::CarouselAlbum) {
            return false;
        }
        if post
            .children
            .as_ref()
            .is_none_or(|children| children.data.len() != want_children)
        {
            return false;
        }
        media_content_matches(post, &text, &topic_tag, reply_to.as_ref(), quoted.as_ref())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{ChildPost, ChildrenData};

    fn base_post(media_type: MediaType) -> Post {
        serde_json::from_value(serde_json::json!({
            "id": "post-1",
            "media_type": serde_json::to_value(media_type).unwrap(),
        }))
        .unwrap()
    }

    fn text_content(text: &str) -> TextPostContent {
        TextPostContent {
            text: text.to_owned(),
            link_attachment: None,
            poll_attachment: None,
            reply_control: None,
            reply_to_id: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            auto_publish_text: false,
            quoted_post_id: None,
            text_entities: None,
            text_attachment: None,
            gif_attachment: None,
            is_ghost_post: false,
            enable_reply_approvals: false,
        }
    }

    fn image_content(text: Option<&str>) -> ImagePostContent {
        ImagePostContent {
            text: text.map(str::to_owned),
            image_url: "https://example.com/img.jpg".into(),
            alt_text: None,
            reply_control: None,
            reply_to_id: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            quoted_post_id: None,
            text_entities: None,
            is_spoiler_media: false,
            enable_reply_approvals: false,
        }
    }

    #[test]
    fn test_should_attempt_publish_recovery_code_10() {
        let mut err = error::new_api_error(10, "GraphMethodException", "", "");
        error::set_error_metadata(&mut err, false, 500, 0);
        assert!(should_attempt_publish_recovery(&err));
    }

    #[test]
    fn test_should_not_attempt_publish_recovery_other_codes() {
        let err = error::new_api_error(1, "Internal error", "", "");
        assert!(!should_attempt_publish_recovery(&err));

        let auth = error::new_authentication_error(190, "Invalid token", "");
        assert!(!should_attempt_publish_recovery(&auth));
    }

    #[test]
    fn test_text_matcher_matches_by_text() {
        let content = text_content("hello world");
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.text = Some("hello world".into());
        assert!(matches(&post));

        post.text = Some("different".into());
        assert!(!matches(&post));
    }

    #[test]
    fn test_text_matcher_accepts_missing_media_type() {
        let content = text_content("hello");
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.media_type = None;
        post.text = Some("hello".into());
        assert!(matches(&post));
    }

    #[test]
    fn test_text_matcher_rejects_wrong_media_type() {
        let content = text_content("hello");
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::Image);
        post.text = Some("hello".into());
        assert!(!matches(&post));
    }

    #[test]
    fn test_text_matcher_rejects_quote_mismatch() {
        let content = text_content("hello");
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.text = Some("hello".into());
        post.is_quote_post = true;
        assert!(!matches(&post), "unexpected quote post must not match");
    }

    #[test]
    fn test_text_matcher_matches_quote_post() {
        let mut content = text_content("hello");
        content.quoted_post_id = Some(PostId::from("quoted-1"));
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.text = Some("hello".into());
        post.is_quote_post = true;
        post.quoted_post = Some(Box::new(base_post(MediaType::TextPost)));
        post.quoted_post.as_mut().unwrap().id = PostId::from("quoted-1");
        assert!(matches(&post));

        post.quoted_post.as_mut().unwrap().id = PostId::from("quoted-other");
        assert!(!matches(&post));
    }

    #[test]
    fn test_text_matcher_reply() {
        let mut content = text_content("reply text");
        content.reply_to_id = Some(PostId::from("parent-1"));
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.text = Some("reply text".into());
        post.is_reply = true;
        post.replied_to = Some(Box::new(base_post(MediaType::TextPost)));
        post.replied_to.as_mut().unwrap().id = PostId::from("parent-1");
        assert!(matches(&post));

        // Same text but a reply to a different parent
        post.replied_to.as_mut().unwrap().id = PostId::from("parent-other");
        assert!(!matches(&post));

        // Same text but not a reply at all
        post.is_reply = false;
        post.replied_to = None;
        assert!(!matches(&post));
    }

    #[test]
    fn test_text_matcher_topic_tag_disambiguates_roots() {
        let mut content = text_content("same text");
        content.topic_tag = Some("rust".into());
        let matches = text_matcher(&content);

        let mut post = base_post(MediaType::TextPost);
        post.text = Some("same text".into());
        post.topic_tag = Some("rust".into());
        assert!(matches(&post));

        post.topic_tag = Some("golang".into());
        assert!(!matches(&post));
    }

    #[test]
    fn test_image_matcher_root_requires_discriminator() {
        // Image-only root with no text/tag/quote has no unique signal —
        // must fail closed even against an otherwise-plausible image post.
        let content = image_content(None);
        let matches = image_matcher(&content);

        let post = base_post(MediaType::Image);
        assert!(!matches(&post));
    }

    #[test]
    fn test_image_matcher_root_with_text() {
        let content = image_content(Some("caption"));
        let matches = image_matcher(&content);

        let mut post = base_post(MediaType::Image);
        post.text = Some("caption".into());
        assert!(matches(&post));

        post.text = None;
        assert!(!matches(&post));
    }

    #[test]
    fn test_image_matcher_reply_requires_discriminator() {
        // Blank-text non-quote reply fails closed even when the parent matches.
        let mut content = image_content(None);
        content.reply_to_id = Some(PostId::from("parent-1"));
        let matches = image_matcher(&content);

        let mut post = base_post(MediaType::Image);
        post.is_reply = true;
        post.replied_to = Some(Box::new(base_post(MediaType::TextPost)));
        post.replied_to.as_mut().unwrap().id = PostId::from("parent-1");
        assert!(!matches(&post));
    }

    #[test]
    fn test_image_matcher_reply_with_text() {
        let mut content = image_content(Some("reply caption"));
        content.reply_to_id = Some(PostId::from("parent-1"));
        let matches = image_matcher(&content);

        let mut post = base_post(MediaType::Image);
        post.text = Some("reply caption".into());
        post.is_reply = true;
        post.replied_to = Some(Box::new(base_post(MediaType::TextPost)));
        post.replied_to.as_mut().unwrap().id = PostId::from("parent-1");
        assert!(matches(&post));
    }

    #[test]
    fn test_video_matcher_accepts_video_and_audio() {
        let content = VideoPostContent {
            text: Some("vid".into()),
            video_url: "https://example.com/v.mp4".into(),
            alt_text: None,
            reply_control: None,
            reply_to_id: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            quoted_post_id: None,
            text_entities: None,
            is_spoiler_media: false,
            enable_reply_approvals: false,
        };
        let matches = video_matcher(&content);

        let mut post = base_post(MediaType::Video);
        post.text = Some("vid".into());
        assert!(matches(&post));

        post.media_type = Some(MediaType::Audio);
        assert!(matches(&post));

        post.media_type = Some(MediaType::Image);
        assert!(!matches(&post));
    }

    #[test]
    fn test_carousel_matcher_children_count() {
        let content = CarouselPostContent {
            text: Some("album".into()),
            children: vec![ContainerId::from("c1"), ContainerId::from("c2")],
            reply_control: None,
            reply_to_id: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            quoted_post_id: None,
            text_entities: None,
            is_spoiler_media: false,
            enable_reply_approvals: false,
        };
        let matches = carousel_matcher(&content);

        let mut post = base_post(MediaType::CarouselAlbum);
        post.text = Some("album".into());
        post.children = Some(ChildrenData {
            data: vec![
                ChildPost {
                    id: PostId::from("p1"),
                },
                ChildPost {
                    id: PostId::from("p2"),
                },
            ],
        });
        assert!(matches(&post));

        // Wrong child count
        post.children.as_mut().unwrap().data.pop();
        assert!(!matches(&post));

        // No children data at all
        post.children = None;
        assert!(!matches(&post));
    }

    #[test]
    fn test_quote_matches() {
        let mut post = base_post(MediaType::TextPost);
        assert!(quote_matches(&post, None));

        post.is_quote_post = true;
        assert!(!quote_matches(&post, None));

        let want = PostId::from("q1");
        // is_quote_post but no embedded quoted_post
        assert!(!quote_matches(&post, Some(&want)));

        post.quoted_post = Some(Box::new(base_post(MediaType::TextPost)));
        post.quoted_post.as_mut().unwrap().id = PostId::from("q1");
        assert!(quote_matches(&post, Some(&want)));
    }

    #[test]
    fn test_discriminator_helpers() {
        assert!(!root_has_unique_discriminator("", "", None));
        assert!(root_has_unique_discriminator("t", "", None));
        assert!(root_has_unique_discriminator("", "tag", None));
        let q = PostId::from("q");
        assert!(root_has_unique_discriminator("", "", Some(&q)));

        assert!(!reply_has_unique_discriminator("", None));
        assert!(reply_has_unique_discriminator("t", None));
        assert!(reply_has_unique_discriminator("", Some(&q)));
    }
}
