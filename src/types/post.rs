use serde::{Deserialize, Serialize};

use super::common::{
    ChildrenData, GifAttachment, MediaType, PollAttachment, PollResult, PostOwner, ReplyControl,
    TextAttachment, TextEntity,
};
use super::ids::{ContainerId, PostId};
use super::pagination::Paging;
use super::time::ThreadsTime;

/// A Threads post with all metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<MediaType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<ThreadsTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PostOwner>,
    #[serde(default)]
    pub is_reply: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_product_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildrenData>,
    #[serde(default)]
    pub is_quote_post: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment_url: Option<String>,
    #[serde(default)]
    pub has_replies: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_audience: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post: Option<Box<Post>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reposted_post: Option<Box<Post>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gif_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_attachment: Option<PollResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_post: Option<Box<Post>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replied_to: Option<Box<Post>>,
    #[serde(default)]
    pub is_reply_owned_by_me: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ghost_post_status: Option<String>,
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
}

/// Generic post content base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
}

/// Content for creating a text post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPostContent {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_attachment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_attachment: Option<PollAttachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default)]
    pub auto_publish_text: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_attachment: Option<TextAttachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gif_attachment: Option<GifAttachment>,
    #[serde(default)]
    pub is_ghost_post: bool,
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating an image post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePostContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub image_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    #[serde(default)]
    pub is_spoiler_media: bool,
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating a video post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPostContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub video_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    #[serde(default)]
    pub is_spoiler_media: bool,
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Content for creating a carousel post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselPostContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Container IDs for the carousel items.
    pub children: Vec<ContainerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_control: Option<ReplyControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlisted_country_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quoted_post_id: Option<PostId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<TextEntity>>,
    #[serde(default)]
    pub is_spoiler_media: bool,
    #[serde(default)]
    pub enable_reply_approvals: bool,
}

/// Paginated response containing multiple posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsResponse {
    pub data: Vec<Post>,
    #[serde(default)]
    pub paging: Paging,
}

/// Paginated response containing reply posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepliesResponse {
    pub data: Vec<Post>,
    #[serde(default)]
    pub paging: Paging,
}

/// Response containing insights data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResponse {
    pub data: Vec<super::insights::Insight>,
}
