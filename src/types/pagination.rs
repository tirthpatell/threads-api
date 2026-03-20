use serde::{Deserialize, Serialize};

use super::common::ApprovalStatus;

/// Pagination information for navigating result sets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Paging {
    /// Cursor-based pagination cursors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursors: Option<PagingCursors>,
    /// Deprecated: use `cursors.before`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Deprecated: use `cursors.after`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Cursor-based pagination cursors.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PagingCursors {
    /// Cursor pointing to the start of the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor pointing to the end of the page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Standard pagination parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationOptions {
    /// Maximum number of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Cursor for previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Options for posts requests with time filtering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostsOptions {
    /// Maximum number of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Cursor for previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Unix timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    /// Unix timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
}

/// Options for replies and conversation requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepliesOptions {
    /// Maximum number of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Cursor for previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// `true` for reverse chronological, `false` for chronological (default: `true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}

/// Options for retrieving pending replies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingRepliesOptions {
    /// Maximum number of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Cursor for previous page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// `true` for reverse chronological, `false` for chronological.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
    /// Filter by approval status: `Pending` or `Ignored`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_status: Option<ApprovalStatus>,
}
