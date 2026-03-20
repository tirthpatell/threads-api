use serde::{Deserialize, Serialize};

use super::ids::UserId;

/// A Threads user profile with app-scoped data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID.
    pub id: UserId,
    /// Username.
    pub username: String,
    /// Display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Maps to `threads_profile_picture_url` in the API.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "threads_profile_picture_url"
    )]
    pub profile_pic_url: Option<String>,
    /// Maps to `threads_biography` in the API.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "threads_biography"
    )]
    pub biography: Option<String>,
    /// Website URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Followers count.
    #[serde(default)]
    pub followers_count: i64,
    /// Total media count.
    #[serde(default)]
    pub media_count: i64,
    /// Whether the user is verified.
    #[serde(default)]
    pub is_verified: bool,
}

/// A public Threads user profile (via `threads_profile_discovery` scope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    /// Username.
    pub username: String,
    /// Display name.
    pub name: String,
    /// Profile picture URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_picture_url: Option<String>,
    /// User biography.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    /// Whether the user is verified.
    #[serde(default)]
    pub is_verified: bool,
    /// Follower count.
    #[serde(default)]
    pub follower_count: i64,
    /// Total likes count.
    #[serde(default)]
    pub likes_count: i64,
    /// Total quotes count.
    #[serde(default)]
    pub quotes_count: i64,
    /// Total replies count.
    #[serde(default)]
    pub replies_count: i64,
    /// Total reposts count.
    #[serde(default)]
    pub reposts_count: i64,
    /// Total views count.
    #[serde(default)]
    pub views_count: i64,
}
