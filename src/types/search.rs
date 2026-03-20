use serde::{Deserialize, Serialize};

use super::common::{SearchMode, SearchType};

/// Options for keyword and topic tag search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Result ordering (top or recent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_type: Option<SearchType>,
    /// Search mode (keyword or tag).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_mode: Option<SearchMode>,
    /// Filter by media type: TEXT, IMAGE, or VIDEO.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Filter by author username (exact match, without @).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_username: Option<String>,
    /// Maximum number of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Unix timestamp (must be >= `MIN_SEARCH_TIMESTAMP`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    /// Unix timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    /// Cursor for previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}
