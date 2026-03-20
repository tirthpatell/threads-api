//! API endpoint implementations grouped by resource.

/// Insights and metrics endpoints.
pub mod insights;
/// Location search and tagging endpoints.
pub mod location;
/// Post creation and publishing endpoints.
pub mod posts;
/// Post deletion endpoints.
pub mod posts_delete;
/// Post retrieval and listing endpoints.
pub mod posts_read;
/// Reply management endpoints.
pub mod replies;
/// Keyword and hashtag search endpoints.
pub mod search;
/// User profile and account endpoints.
pub mod users;
