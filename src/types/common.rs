use serde::{Deserialize, Serialize};

use super::ids::{ContainerId, PostId, UserId};
use super::time::ThreadsTime;

/// Controls who can reply to a post.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplyControl {
    /// Anyone can reply.
    #[serde(rename = "everyone")]
    Everyone,
    /// Only accounts the poster follows can reply.
    #[serde(rename = "accounts_you_follow")]
    AccountsYouFollow,
    /// Only mentioned accounts can reply.
    #[serde(rename = "mentioned_only")]
    MentionedOnly,
    /// Only the parent post author can reply.
    #[serde(rename = "parent_post_author_only")]
    ParentPostAuthorOnly,
    /// Only followers can reply.
    #[serde(rename = "followers_only")]
    FollowersOnly,
}

/// Approval status for pending replies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Reply is awaiting approval.
    #[serde(rename = "pending")]
    Pending,
    /// Reply has been ignored.
    #[serde(rename = "ignored")]
    Ignored,
}

/// Search result ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    /// Order results by relevance.
    #[serde(rename = "TOP")]
    Top,
    /// Order results by recency.
    #[serde(rename = "RECENT")]
    Recent,
}

/// Search mode selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    /// Search by keyword.
    #[serde(rename = "KEYWORD")]
    Keyword,
    /// Search by hashtag.
    #[serde(rename = "TAG")]
    Tag,
}

/// GIF provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GifProvider {
    /// Tenor GIF provider. Deprecated: Tenor API will be sunsetted by March 31, 2026.
    #[serde(rename = "TENOR")]
    Tenor,
    /// Giphy GIF provider.
    #[serde(rename = "GIPHY")]
    Giphy,
}

/// Media type for posts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// Text-only post.
    #[serde(rename = "TEXT")]
    Text,
    /// Single image post.
    #[serde(rename = "IMAGE")]
    Image,
    /// Single video post.
    #[serde(rename = "VIDEO")]
    Video,
    /// Carousel post (multiple media items).
    #[serde(rename = "CAROUSEL")]
    Carousel,
    /// Carousel album container.
    #[serde(rename = "CAROUSEL_ALBUM")]
    CarouselAlbum,
    /// Repost facade.
    #[serde(rename = "REPOST_FACADE")]
    RepostFacade,
}

/// Poll options when creating a post with a poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAttachment {
    /// First poll option text.
    pub option_a: String,
    /// Second poll option text.
    pub option_b: String,
    /// Third poll option (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_c: Option<String>,
    /// Fourth poll option (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_d: Option<String>,
}

/// Poll results and voting statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResult {
    /// First poll option text.
    pub option_a: String,
    /// Second poll option text.
    pub option_b: String,
    /// Third poll option (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_c: Option<String>,
    /// Fourth poll option (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_d: Option<String>,
    /// Vote percentage for option A.
    pub option_a_votes_percentage: f64,
    /// Vote percentage for option B.
    pub option_b_votes_percentage: f64,
    /// Vote percentage for option C.
    #[serde(default)]
    pub option_c_votes_percentage: f64,
    /// Vote percentage for option D.
    #[serde(default)]
    pub option_d_votes_percentage: f64,
    /// Total number of votes cast.
    pub total_votes: i64,
    /// When the poll expires.
    pub expiration_timestamp: ThreadsTime,
}

/// Spoiler entity within text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntity {
    /// Entity type identifier.
    pub entity_type: String,
    /// Character offset where the entity starts.
    pub offset: usize,
    /// Character length of the entity.
    pub length: usize,
}

/// Text attachment with optional styling (up to 10,000 chars).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAttachment {
    /// Plain text content.
    pub plaintext: String,
    /// Optional link attachment URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment_url: Option<String>,
    /// Optional styling information for text ranges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_with_styling_info: Option<Vec<TextStylingInfo>>,
}

/// Styling information for a range of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStylingInfo {
    /// Character offset where styling starts.
    pub offset: usize,
    /// Character length of the styled range.
    pub length: usize,
    /// List of styling tags applied.
    pub styling_info: Vec<String>,
}

/// GIF attachment for text posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifAttachment {
    /// GIF identifier from the provider.
    pub gif_id: String,
    /// GIF provider (Tenor or Giphy).
    pub provider: GifProvider,
}

/// Owner of a post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostOwner {
    /// User ID of the owner.
    pub id: UserId,
}

/// Children data for carousel posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenData {
    /// List of child posts.
    pub data: Vec<ChildPost>,
}

/// A child post in a carousel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPost {
    /// Child post ID.
    pub id: PostId,
}

/// Status of a media container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    /// Container ID.
    pub id: ContainerId,
    /// Current container status.
    pub status: String,
    /// Error message if status is ERROR.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Content for creating a repost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepostContent {
    /// ID of the post to repost.
    pub post_id: PostId,
}

/// Quota configuration for a specific operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Total quota allowed.
    pub quota_total: i64,
    /// Quota window duration in seconds.
    pub quota_duration: i64,
}

/// Current API quota usage and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingLimits {
    /// Post creation quota used.
    pub quota_usage: i64,
    /// Post creation quota config.
    pub config: QuotaConfig,
    /// Reply quota used.
    pub reply_quota_usage: i64,
    /// Reply quota config.
    pub reply_config: QuotaConfig,
    /// Delete quota used.
    pub delete_quota_usage: i64,
    /// Delete quota config.
    pub delete_config: QuotaConfig,
    /// Location search quota used.
    pub location_search_quota_usage: i64,
    /// Location search quota config.
    pub location_search_config: QuotaConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reply_control_serde() {
        let rc = ReplyControl::AccountsYouFollow;
        let json = serde_json::to_string(&rc).unwrap();
        assert_eq!(json, r#""accounts_you_follow""#);
        let back: ReplyControl = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ReplyControl::AccountsYouFollow);
    }

    #[test]
    fn test_media_type_serde() {
        let mt = MediaType::Carousel;
        let json = serde_json::to_string(&mt).unwrap();
        assert_eq!(json, r#""CAROUSEL""#);
        let back: MediaType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, MediaType::Carousel);
    }

    #[test]
    fn test_search_type_serde() {
        let st = SearchType::Recent;
        let json = serde_json::to_string(&st).unwrap();
        assert_eq!(json, r#""RECENT""#);
    }

    #[test]
    fn test_search_mode_serde() {
        let sm = SearchMode::Tag;
        let json = serde_json::to_string(&sm).unwrap();
        assert_eq!(json, r#""TAG""#);
    }

    #[test]
    fn test_gif_provider_serde() {
        let gp = GifProvider::Giphy;
        let json = serde_json::to_string(&gp).unwrap();
        assert_eq!(json, r#""GIPHY""#);
    }

    #[test]
    fn test_approval_status_serde() {
        let s = ApprovalStatus::Pending;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#""pending""#);
    }

    #[test]
    fn test_poll_attachment_serde() {
        let poll = PollAttachment {
            option_a: "Yes".into(),
            option_b: "No".into(),
            option_c: None,
            option_d: None,
        };
        let json = serde_json::to_string(&poll).unwrap();
        assert!(!json.contains("option_c"));
        let back: PollAttachment = serde_json::from_str(&json).unwrap();
        assert_eq!(back.option_a, "Yes");
    }

    #[test]
    fn test_container_status_serde() {
        let cs = ContainerStatus {
            id: ContainerId::from("123"),
            status: "FINISHED".into(),
            error_message: None,
        };
        let json = serde_json::to_string(&cs).unwrap();
        assert!(!json.contains("error_message"));
    }
}
