#![warn(missing_docs)]

//! Rust client library for the [Meta Threads API](https://developers.facebook.com/docs/threads).
//!
//! # Features
//!
//! - Full Threads API coverage: posts, replies, insights, user profiles, search, and location tagging
//! - OAuth 2.0 authentication with short-lived and long-lived token exchange
//! - Automatic rate limiting and retry with exponential backoff
//! - Cursor-based pagination helpers
//! - Strongly typed request/response models
//! - Pluggable token storage
//!
//! # Quick start
//!
//! ```rust,no_run
//! use threads_rs::client::{Config, Client};
//!
//! # async fn run() -> threads_rs::Result<()> {
//! let config = Config::new("client-id", "client-secret", "https://example.com/cb");
//! let client = Client::with_token(config, "ACCESS_TOKEN").await?;
//!
//! let me = client.get_me().await?;
//! println!("Logged in as @{}", me.username);
//! # Ok(())
//! # }
//! ```

/// Application constants (API base URL, version, timeouts).
pub mod constants;
/// Error types and helpers.
pub mod error;
/// Request and response model types.
pub mod types;

/// API endpoint implementations.
pub mod api;
/// OAuth 2.0 authentication flows.
pub mod auth;
/// Client configuration and construction.
pub mod client;
/// HTTP transport with retry and rate-limit integration.
pub mod http;
/// Cursor-based pagination utilities.
pub mod pagination;
/// Rate limiter for API request throttling.
pub mod rate_limit;
/// Input validation helpers.
pub mod validation;

// Re-export primary types for convenience
pub use error::{Error, Result};
pub use types::*;
