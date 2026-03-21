# threads-api

Rust client library for the [Meta Threads API](https://developers.facebook.com/docs/threads).

> Rust port of [threads-go](https://github.com/tirthpatell/threads-go).

## Features

- Full Threads API coverage: posts, replies, insights, user profiles, search, and location tagging
- OAuth 2.0 authentication with short-lived and long-lived token exchange
- Automatic rate limiting and retry with exponential backoff
- Cursor-based pagination helpers
- Strongly typed request/response models
- Pluggable token storage

## Requirements

- Rust 1.85+
- A [Meta Threads API](https://developers.facebook.com/docs/threads/get-started) app with client ID, secret, and redirect URI

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
meta-threads = { git = "https://github.com/tirthpatell/threads-api" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick start

```rust,no_run
use meta_threads::client::{Config, Client};

#[tokio::main]
async fn main() -> meta_threads::Result<()> {
    let config = Config::new("client-id", "client-secret", "https://example.com/cb");
    let client = Client::with_token(config, "YOUR_ACCESS_TOKEN").await?;

    let me = client.get_me().await?;
    println!("Logged in as @{}", me.username);

    Ok(())
}
```

## Authentication

The library supports the full Threads OAuth 2.0 flow:

```rust,no_run
use meta_threads::client::{Config, Client};

# async fn run() -> meta_threads::Result<()> {
let config = Config::new("client-id", "client-secret", "https://example.com/cb");
let client = Client::new(config).await?;

// 1. Generate the authorization URL
let (auth_url, state) = client.get_auth_url(&[]);
// Redirect user to auth_url, verify state on callback

// 2. Exchange the authorization code for a short-lived token
client.exchange_code_for_token("AUTH_CODE").await?;

// 3. Convert to a long-lived token (60 days)
client.get_long_lived_token().await?;

// 4. Refresh before expiry
client.refresh_token().await?;
# Ok(())
# }
```

You can also create a client from environment variables:

```rust,no_run
# async fn run() -> meta_threads::Result<()> {
// Reads THREADS_CLIENT_ID, THREADS_CLIENT_SECRET, THREADS_REDIRECT_URI
let client = meta_threads::client::Client::from_env().await?;
# Ok(())
# }
```

## API coverage

### Posts

| Method | Description |
|--------|-------------|
| `create_text_post` | Create a text post |
| `create_image_post` | Create an image post |
| `create_video_post` | Create a video post |
| `create_carousel_post` | Create a carousel post (2-20 items) |
| `create_quote_post` | Quote an existing post |
| `repost_post` | Repost an existing post |
| `get_post` | Get a single post by ID |
| `get_user_posts` | Get a user's posts (paginated) |
| `get_user_mentions` | Get posts mentioning the user |
| `get_user_ghost_posts` | Get ephemeral/ghost posts |
| `delete_post` | Delete a post |
| `get_container_status` | Check media upload status |
| `get_publishing_limits` | Get current quota usage |

### Replies

| Method | Description |
|--------|-------------|
| `reply_to_post` | Reply to a post with text |
| `create_reply` | Reply with full content options |
| `get_replies` | Get replies to a post (paginated) |
| `get_conversation` | Get the full conversation thread |
| `get_pending_replies` | Get replies awaiting approval |
| `approve_pending_reply` | Approve a pending reply |
| `ignore_pending_reply` | Ignore a pending reply |
| `hide_reply` / `unhide_reply` | Toggle reply visibility |

### Users

| Method | Description |
|--------|-------------|
| `get_me` | Get the authenticated user's profile |
| `get_user` | Get a user by ID |
| `get_user_with_fields` | Get a user with custom fields |
| `lookup_public_profile` | Look up a public profile by username |
| `get_public_profile_posts` | Get a public user's posts |
| `get_user_replies` | Get a user's replies |

### Insights

| Method | Description |
|--------|-------------|
| `get_post_insights` | Get metrics for a post |
| `get_account_insights` | Get account-level metrics |
| `get_post_insights_with_options` | Post insights with typed options |
| `get_account_insights_with_options` | Account insights with typed options |

### Search & Location

| Method | Description |
|--------|-------------|
| `keyword_search` | Search posts by keyword or hashtag |
| `search_locations` | Search for taggable locations |
| `get_location` | Get location details by ID |

### Pagination

Built-in iterators for paginated endpoints:

```rust,no_run
use meta_threads::client::{Config, Client};
use meta_threads::pagination::PostIterator;
use meta_threads::types::ids::UserId;

# async fn run() -> meta_threads::Result<()> {
# let config = Config::new("id", "secret", "https://example.com/cb");
# let client = Client::with_token(config, "TOKEN").await?;
let user_id = UserId::from("me");
let mut iter = PostIterator::new(&client, user_id, None);

while let Some(page) = iter.next().await? {
    for post in &page.data {
        println!("{}: {}", post.id, post.text.as_deref().unwrap_or(""));
    }
}
# Ok(())
# }
```

`ReplyIterator` and `SearchIterator` work the same way.

## Configuration

All fields on `Config` are public and can be customized:

```rust
use meta_threads::client::Config;
use std::time::Duration;

let mut config = Config::new("client-id", "client-secret", "https://example.com/cb");
config.http_timeout = Duration::from_secs(60);
config.debug = true;
```

Environment variable configuration is also supported via `Config::from_env()`:

| Variable | Required | Description |
|----------|----------|-------------|
| `THREADS_CLIENT_ID` | Yes | OAuth client ID |
| `THREADS_CLIENT_SECRET` | Yes | OAuth client secret |
| `THREADS_REDIRECT_URI` | Yes | OAuth redirect URI |
| `THREADS_SCOPES` | No | Comma-separated scopes |
| `THREADS_HTTP_TIMEOUT` | No | Timeout in seconds |
| `THREADS_BASE_URL` | No | API base URL override |
| `THREADS_MAX_RETRIES` | No | Max retry attempts |
| `THREADS_DEBUG` | No | Enable debug logging |

## Error handling

All API methods return `meta_threads::Result<T>`. Errors are strongly typed:

```rust,no_run
use meta_threads::Error;

# async fn run(client: &meta_threads::client::Client) {
# let post_id = meta_threads::types::ids::PostId::from("123");
match client.get_post(&post_id).await {
    Ok(post) => println!("{:?}", post),
    Err(Error::Authentication { message, .. }) => eprintln!("Auth failed: {message}"),
    Err(Error::RateLimit { retry_after, .. }) => eprintln!("Rate limited, retry in {retry_after:?}"),
    Err(Error::Validation { field, message, .. }) => eprintln!("Invalid {field}: {message}"),
    Err(e) => eprintln!("Error: {e}"),
}
# }
```

## License

MIT
