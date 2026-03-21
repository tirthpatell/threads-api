use std::time::Duration;

// --- Text limits ---

/// Maximum characters for post text.
pub const MAX_TEXT_LENGTH: usize = 500;

/// Maximum characters for text attachment plaintext.
pub const MAX_TEXT_ATTACHMENT_LENGTH: usize = 10_000;

/// Maximum text spoiler entities per post.
pub const MAX_TEXT_ENTITIES: usize = 10;

/// Maximum number of links in a post.
pub const MAX_LINKS: usize = 5;

// --- Pagination limits ---

/// Maximum posts per API request.
pub const MAX_POSTS_PER_REQUEST: usize = 100;

/// Default number of posts if not specified.
pub const DEFAULT_POSTS_LIMIT: usize = 25;

// --- Carousel limits ---

/// Minimum items in a carousel.
pub const MIN_CAROUSEL_ITEMS: usize = 2;

/// Maximum items in a carousel.
pub const MAX_CAROUSEL_ITEMS: usize = 20;

// --- Reply processing ---

/// Recommended delay before publishing reply.
pub const REPLY_PUBLISH_DELAY: Duration = Duration::from_secs(10);

// --- Poll limits ---

/// Minimum number of poll options.
pub const MIN_POLL_OPTIONS: usize = 2;

/// Maximum number of poll options.
pub const MAX_POLL_OPTIONS: usize = 4;

/// Maximum characters per poll option.
pub const MAX_POLL_OPTION_LENGTH: usize = 25;

// --- Topic tag limits ---

/// Maximum characters for a topic tag.
pub const MAX_TOPIC_TAG_LENGTH: usize = 50;

// --- Alt text limits ---

/// Maximum characters for alt text on media.
pub const MAX_ALT_TEXT_LENGTH: usize = 1000;

// --- Search constraints ---

/// Minimum timestamp for search queries (July 5, 2023).
pub const MIN_SEARCH_TIMESTAMP: i64 = 1_688_540_400;

// --- Library version ---

/// Library version string from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// --- HTTP client defaults ---

/// Default HTTP request timeout.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);

// --- API Endpoints ---

/// Base URL for the Threads Graph API.
pub const BASE_API_URL: &str = "https://graph.threads.net";

// --- Field sets for API requests ---

/// Default field set for post queries.
pub const POST_EXTENDED_FIELDS: &str = "id,media_product_type,media_type,media_url,permalink,owner,username,text,timestamp,shortcode,thumbnail_url,children,is_quote_post,alt_text,link_attachment_url,has_replies,reply_audience,quoted_post,reposted_post,gif_url,is_verified,profile_picture_url,poll_attachment,topic_tag,is_spoiler_media,text_entities,text_attachment,location_id,location,allowlisted_country_codes,ghost_post_status,ghost_post_expiration_timestamp,is_reply,root_post,replied_to,is_reply_owned_by_me,hide_status,reply_approval_status";

/// Field set for ghost/ephemeral post queries.
pub const GHOST_POST_FIELDS: &str = "id,media_product_type,media_type,media_url,permalink,owner,username,text,timestamp,shortcode,thumbnail_url,ghost_post_status,ghost_post_expiration_timestamp";

/// Field set for user profile queries.
pub const USER_PROFILE_FIELDS: &str = "id,username,name,threads_profile_picture_url,threads_biography,is_verified,is_eligible_for_geo_gating,recently_searched_keywords";

/// Field set for reply queries (`/{media-id}/replies` and `/{media-id}/conversation`).
pub const REPLY_FIELDS: &str = "id,media_product_type,media_type,media_url,permalink,username,text,timestamp,shortcode,thumbnail_url,children,is_quote_post,has_replies,root_post,replied_to,is_reply,is_reply_owned_by_me,reply_audience,quoted_post,reposted_post,gif_url,hide_status,topic_tag,is_verified,profile_picture_url";

/// Field set for pending reply queries (`/{media-id}/pending_replies`).
/// Includes `reply_approval_status` and `alt_text` which are only valid for pending replies.
pub const PENDING_REPLY_FIELDS: &str = "id,media_product_type,media_type,media_url,permalink,username,text,timestamp,shortcode,thumbnail_url,children,is_quote_post,has_replies,root_post,replied_to,is_reply,is_reply_owned_by_me,reply_audience,quoted_post,reposted_post,gif_url,alt_text,hide_status,topic_tag,is_verified,profile_picture_url,reply_approval_status";

/// Field set for public user profile queries.
pub const PUBLIC_USER_FIELDS: &str = "username,name,profile_picture_url,biography,is_verified,follower_count,likes_count,quotes_count,replies_count,reposts_count,views_count";

/// Field set for container status queries.
pub const CONTAINER_STATUS_FIELDS: &str = "id,status,error_message";

/// Field set for location queries.
pub const LOCATION_FIELDS: &str = "id,address,name,city,country,latitude,longitude,postal_code";

/// Field set for publishing limit queries.
pub const PUBLISHING_LIMIT_FIELDS: &str = "quota_usage,config,reply_quota_usage,reply_config,delete_quota_usage,delete_config,location_search_quota_usage,location_search_config";

// --- Container status values ---

/// Container status: upload in progress.
pub const CONTAINER_STATUS_IN_PROGRESS: &str = "IN_PROGRESS";
/// Container status: upload finished, ready to publish.
pub const CONTAINER_STATUS_FINISHED: &str = "FINISHED";
/// Container status: published.
pub const CONTAINER_STATUS_PUBLISHED: &str = "PUBLISHED";
/// Container status: error occurred.
pub const CONTAINER_STATUS_ERROR: &str = "ERROR";
/// Container status: expired.
pub const CONTAINER_STATUS_EXPIRED: &str = "EXPIRED";

/// Maximum number of polling attempts for container status.
/// Documentation recommends querying once per minute for no more than 5 minutes.
pub const DEFAULT_CONTAINER_POLL_MAX_ATTEMPTS: u32 = 5;

/// Interval between polling attempts.
/// Documentation recommends querying once per minute.
pub const DEFAULT_CONTAINER_POLL_INTERVAL: Duration = Duration::from_secs(60);

// --- Media types ---

/// Media type constant for text posts.
pub const MEDIA_TYPE_TEXT: &str = "TEXT";
/// Media type constant for image posts.
pub const MEDIA_TYPE_IMAGE: &str = "IMAGE";
/// Media type constant for video posts.
pub const MEDIA_TYPE_VIDEO: &str = "VIDEO";
/// Media type constant for carousel posts.
pub const MEDIA_TYPE_CAROUSEL: &str = "CAROUSEL";
/// Media type constant for audio posts.
pub const MEDIA_TYPE_AUDIO: &str = "AUDIO";
/// Media type constant for repost facade.
pub const MEDIA_TYPE_REPOST_FACADE: &str = "REPOST_FACADE";
/// Media type constant for text post responses.
pub const MEDIA_TYPE_RESPONSE_TEXT: &str = "TEXT_POST";
/// Media type constant for carousel album responses.
pub const MEDIA_TYPE_RESPONSE_CAROUSEL: &str = "CAROUSEL_ALBUM";

// --- Error messages ---

/// Validation error message: post ID required.
pub const ERR_EMPTY_POST_ID: &str = "Post ID is required";
/// Validation error message: user ID required.
pub const ERR_EMPTY_USER_ID: &str = "User ID is required";
/// Validation error message: container ID required.
pub const ERR_EMPTY_CONTAINER_ID: &str = "Container ID is required";
/// Validation error message: search query required.
pub const ERR_EMPTY_SEARCH_QUERY: &str = "Search query is required";

// --- API Error Codes ---

/// Returned when a post contains more than 5 unique links.
pub const ERR_CODE_LINK_LIMIT_EXCEEDED: &str = "THREADS_API__LINK_LIMIT_EXCEEDED";
/// Returned when a feature is not available.
pub const ERR_CODE_FEATURE_NOT_AVAILABLE: &str = "THREADS_API__FEATURE_NOT_AVAILABLE";
/// Returned when geo-gating country codes are invalid.
pub const ERR_CODE_GEO_GATING_INVALID_COUNTRY_CODES: &str =
    "THREADS_API__GEO_GATING_INVALID_COUNTRY_CODES";

// --- Container error messages ---

/// Container error: failed downloading video.
pub const CONTAINER_ERR_FAILED_DOWNLOADING_VIDEO: &str = "FAILED_DOWNLOADING_VIDEO";
/// Container error: failed processing audio.
pub const CONTAINER_ERR_FAILED_PROCESSING_AUDIO: &str = "FAILED_PROCESSING_AUDIO";
/// Container error: failed processing video.
pub const CONTAINER_ERR_FAILED_PROCESSING_VIDEO: &str = "FAILED_PROCESSING_VIDEO";
/// Container error: invalid aspect ratio.
pub const CONTAINER_ERR_INVALID_ASPECT_RATIO: &str = "INVALID_ASPEC_RATIO";
/// Container error: invalid bit rate.
pub const CONTAINER_ERR_INVALID_BIT_RATE: &str = "INVALID_BIT_RATE";
/// Container error: invalid duration.
pub const CONTAINER_ERR_INVALID_DURATION: &str = "INVALID_DURATION";
/// Container error: invalid frame rate.
pub const CONTAINER_ERR_INVALID_FRAME_RATE: &str = "INVALID_FRAME_RATE";
/// Container error: invalid audio channels.
pub const CONTAINER_ERR_INVALID_AUDIO_CHANNELS: &str = "INVALID_AUDIO_CHANNELS";
/// Container error: invalid audio channel layout.
pub const CONTAINER_ERR_INVALID_AUDIO_CHANNEL_LAYOUT: &str = "INVALID_AUDIO_CHANNEL_LAYOUT";
/// Container error: unknown error.
pub const CONTAINER_ERR_UNKNOWN: &str = "UNKNOWN";
