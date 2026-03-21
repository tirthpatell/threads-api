use serde::{Deserialize, Serialize};

use super::common::{
    ChildrenData, GifAttachment, HideStatus, MediaType, PollAttachment, PollResult, PostOwner,
    ReplyAudience, ReplyControl, TextAttachment, TextEntitiesResponse, TextEntity,
};
use super::ids::{ContainerId, PostId};
use super::location::Location;
use super::pagination::Paging;
use super::time::ThreadsTime;

/// A Threads post with all metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Post ID.
    pub id: PostId,
    /// Post text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Type of media attached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<MediaType>,
    /// URL of the attached media.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    /// Permanent link to the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    /// When the post was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<ThreadsTime>,
    /// Author's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Post owner info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PostOwner>,
    /// Whether this post is a reply.
    #[serde(default)]
    pub is_reply: bool,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    /// Product type classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_product_type: Option<String>,
    /// Post shortcode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcode: Option<String>,
    /// Thumbnail URL for video posts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// Alt text for media.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Child posts for carousels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildrenData>,
    /// Whether this is a quote post.
    #[serde(default)]
    pub is_quote_post: bool,
    /// Attached link URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment_url: Option<String>,
    /// Whether the post has replies.
    #[serde(default)]
    pub has_replies: bool,
    /// Reply audience setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_audience: Option<ReplyAudience>,
    /// The quoted post, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post: Option<Box<Post>>,
    /// The reposted post, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reposted_post: Option<Box<Post>>,
    /// GIF URL if attached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gif_url: Option<String>,
    /// Poll results, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_attachment: Option<PollResult>,
    /// Root post of the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_post: Option<Box<Post>>,
    /// Post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replied_to: Option<Box<Post>>,
    /// Whether the reply is owned by the authenticated user.
    #[serde(default)]
    pub is_reply_owned_by_me: bool,
    /// Hide status of the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_status: Option<HideStatus>,
    /// Topic tag for the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    /// Ghost post status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ghost_post_status: Option<String>,
    /// When the ghost post expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ghost_post_expiration_timestamp: Option<ThreadsTime>,
    /// Whether the post author is verified on Threads.
    #[serde(default)]
    pub is_verified: bool,
    /// Profile picture URL of the post author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_picture_url: Option<String>,
    /// Approval status of a pending reply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_approval_status: Option<String>,
    /// Whether the media is marked as a spoiler.
    #[serde(default)]
    pub is_spoiler_media: bool,
    /// Text entities in the post content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<TextEntitiesResponse>,
    /// Long-form text attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_attachment: Option<TextAttachment>,
    /// Allowlisted country codes for geo-gating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    /// Location ID tagged in the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// Location details tagged in the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

/// Generic post content base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostContent {
    /// Post text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Media type string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
}

/// Content for creating a text post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPostContent {
    /// Post text content.
    pub text: String,
    /// Link attachment URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment: Option<String>,
    /// Poll attachment options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_attachment: Option<PollAttachment>,
    /// Reply control setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    /// Topic tag for the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    /// Allowlisted country codes for visibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    /// Location ID to tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// Whether to auto-publish the text post.
    #[serde(default)]
    pub auto_publish_text: bool,
    /// ID of the post being quoted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    /// Text entities for spoiler styling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    /// Long-form text attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_attachment: Option<TextAttachment>,
    /// GIF attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gif_attachment: Option<GifAttachment>,
    /// Whether this is a ghost post.
    #[serde(default)]
    pub is_ghost_post: bool,
    /// Whether reply approvals are enabled.
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating an image post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePostContent {
    /// Post text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Image URL.
    pub image_url: String,
    /// Alt text for the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Reply control setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    /// Topic tag for the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    /// Allowlisted country codes for visibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    /// Location ID to tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// ID of the post being quoted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    /// Text entities for spoiler styling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    /// Whether the media is marked as a spoiler.
    #[serde(default)]
    pub is_spoiler_media: bool,
    /// Whether reply approvals are enabled.
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating a video post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPostContent {
    /// Post text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Video URL.
    pub video_url: String,
    /// Alt text for the video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Reply control setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    /// Topic tag for the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    /// Allowlisted country codes for visibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    /// Location ID to tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// ID of the post being quoted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    /// Text entities for spoiler styling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    /// Whether the media is marked as a spoiler.
    #[serde(default)]
    pub is_spoiler_media: bool,
    /// Whether reply approvals are enabled.
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating a carousel post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselPostContent {
    /// Post text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Container IDs for the carousel items.
    pub children: Vec<ContainerId>,
    /// Reply control setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    /// ID of the post being replied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    /// Topic tag for the post.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    /// Allowlisted country codes for visibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    /// Location ID to tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    /// ID of the post being quoted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    /// Text entities for spoiler styling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    /// Whether the media is marked as a spoiler.
    #[serde(default)]
    pub is_spoiler_media: bool,
    /// Whether reply approvals are enabled.
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Paginated response containing multiple posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsResponse {
    /// List of posts.
    pub data: Vec<Post>,
    /// Pagination info.
    #[serde(default)]
    pub paging: Paging,
}

/// Paginated response containing reply posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepliesResponse {
    /// List of reply posts.
    pub data: Vec<Post>,
    /// Pagination info.
    #[serde(default)]
    pub paging: Paging,
}

/// Response containing insights data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResponse {
    /// List of insight metrics.
    pub data: Vec<super::insights::Insight>,
}
