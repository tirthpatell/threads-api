use serde::{Deserialize, Serialize};

use super::time::ThreadsTime;

/// Controls who can reply to a post.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplyControl {
    #[serde(rename = "everyone")]
    Everyone,
    #[serde(rename = "accounts_you_follow")]
    AccountsYouFollow,
    #[serde(rename = "mentioned_only")]
    MentionedOnly,
    #[serde(rename = "parent_post_author_only")]
    ParentPostAuthorOnly,
    #[serde(rename = "followers_only")]
    FollowersOnly,
}

/// Approval status for pending replies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "ignored")]
    Ignored,
}

/// Search result ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    #[serde(rename = "TOP")]
    Top,
    #[serde(rename = "RECENT")]
    Recent,
}

/// Search mode selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    #[serde(rename = "KEYWORD")]
    Keyword,
    #[serde(rename = "TAG")]
    Tag,
}

/// GIF provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GifProvider {
    /// Deprecated: Tenor API will be sunsetted by March 31, 2026.
    #[serde(rename = "TENOR")]
    Tenor,
    #[serde(rename = "GIPHY")]
    Giphy,
}

/// Media type for posts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    #[serde(rename = "TEXT")]
    Text,
    #[serde(rename = "IMAGE")]
    Image,
    #[serde(rename = "VIDEO")]
    Video,
    #[serde(rename = "CAROUSEL")]
    Carousel,
}

/// Poll options when creating a post with a poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAttachment {
    pub option_a: String,
    pub option_b: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_c: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_d: Option<String>,
}

/// Poll results and voting statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResult {
    pub option_a: String,
    pub option_b: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_c: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_d: Option<String>,
    pub option_a_votes_percentage: f64,
    pub option_b_votes_percentage: f64,
    #[serde(default)]
    pub option_c_votes_percentage: f64,
    #[serde(default)]
    pub option_d_votes_percentage: f64,
    pub total_votes: i64,
    pub expiration_timestamp: ThreadsTime,
}

/// Spoiler entity within text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntity {
    pub entity_type: String,
    pub offset: usize,
    pub length: usize,
}

/// Text attachment with optional styling (up to 10,000 chars).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAttachment {
    pub plaintext: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_with_styling_info: Option<Vec<TextStylingInfo>>,
}

/// Styling information for a range of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStylingInfo {
    pub offset: usize,
    pub length: usize,
    pub styling_info: Vec<String>,
}

/// GIF attachment for text posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifAttachment {
    pub gif_id: String,
    pub provider: GifProvider,
}

/// Owner of a post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostOwner {
    pub id: String,
}

/// Children data for carousel posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenData {
    pub data: Vec<ChildPost>,
}

/// A child post in a carousel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPost {
    pub id: String,
}

/// Status of a media container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Content for creating a repost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepostContent {
    pub post_id: String,
}

/// Quota configuration for a specific operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    pub quota_total: i64,
    pub quota_duration: i64,
}

/// Current API quota usage and limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingLimits {
    pub quota_usage: i64,
    pub config: QuotaConfig,
    pub reply_quota_usage: i64,
    pub reply_config: QuotaConfig,
    pub delete_quota_usage: i64,
    pub delete_config: QuotaConfig,
    pub location_search_quota_usage: i64,
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
            id: "123".into(),
            status: "FINISHED".into(),
            error_message: None,
        };
        let json = serde_json::to_string(&cs).unwrap();
        assert!(!json.contains("error_message"));
    }
}
