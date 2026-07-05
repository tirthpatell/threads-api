//! Integration tests for the fixes ported from threads-go:
//! - `debug_token` calling Meta with the app access token
//! - `Client::with_token` falling back to `/me` when `debug_token` fails
//! - auth error codes (190/102) never being retried
//! - recovery from `/threads_publish` HTTP 5xx + code 10 false failures

use std::time::Duration;

use chrono::Utc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use threads_rs::client::{Client, Config, TokenInfo};
use threads_rs::types::TextPostContent;

fn test_config(base_url: &str) -> Config {
    let mut config = Config::new(
        "test-client-id",
        "test-secret",
        "https://example.com/callback",
    );
    config.base_url = base_url.to_owned();
    config.retry_config.max_retries = 0;
    config.retry_config.initial_delay = Duration::from_millis(1);
    config
}

async fn authenticated_client(base_url: &str) -> Client {
    let client = Client::new(test_config(base_url)).await.unwrap();
    client
        .set_token_info(TokenInfo {
            access_token: "user-token".into(),
            token_type: "bearer".into(),
            expires_at: Utc::now() + chrono::Duration::days(30),
            user_id: "user-1".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    client
}

fn text_content(text: &str) -> TextPostContent {
    TextPostContent {
        text: text.to_owned(),
        link_attachment: None,
        poll_attachment: None,
        reply_control: None,
        reply_to_id: None,
        topic_tag: None,
        allowlisted_country_codes: None,
        location_id: None,
        auto_publish_text: false,
        quoted_post_id: None,
        text_entities: None,
        text_attachment: None,
        gif_attachment: None,
        is_ghost_post: false,
        enable_reply_approvals: false,
    }
}

// ---------------------------------------------------------------------------
// debug_token app access token (Go #28)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn debug_token_calls_meta_with_app_access_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/debug_token"))
        .and(query_param("input_token", "user-token"))
        .and(query_param("access_token", "TH|test-client-id|test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "is_valid": true,
                "expires_at": 1893456000i64,
                "issued_at": 1700000000i64,
                "scopes": ["threads_basic"],
                "user_id": "user-1"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    let resp = client.debug_token("user-token").await.unwrap();
    assert!(resp.data.is_valid);
    assert_eq!(resp.data.user_id, "user-1");
}

#[tokio::test]
async fn debug_token_defaults_input_token_to_stored_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/debug_token"))
        .and(query_param("input_token", "user-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "is_valid": true,
                "expires_at": 1893456000i64,
                "issued_at": 1700000000i64,
                "scopes": ["threads_basic"],
                "user_id": "user-1"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    client.debug_token("").await.unwrap();
}

// ---------------------------------------------------------------------------
// with_token /me fallback (Go #29)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn with_token_falls_back_to_me_when_debug_token_fails() {
    let server = MockServer::start().await;

    // debug_token fails the way graph.threads.net does for dev-mode apps:
    // HTTP 500 with an auth error body.
    Mock::given(method("GET"))
        .and(path("/debug_token"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {"message": "Invalid OAuth access token", "code": 190}
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/me"))
        .and(query_param("fields", "id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "me-42"})))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::with_token(test_config(&server.uri()), "some-token")
        .await
        .unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.user_id, "me-42");
    assert_eq!(info.access_token, "some-token");
    // 60-day expiry bootstrap
    let days_left = (info.expires_at - Utc::now()).num_days();
    assert!((58..=60).contains(&days_left), "days_left = {days_left}");
}

#[tokio::test]
async fn with_token_fails_when_both_debug_token_and_me_fail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/debug_token"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {"message": "Invalid OAuth access token", "code": 190}
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/me"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {"message": "Invalid OAuth access token", "code": 190}
        })))
        .mount(&server)
        .await;

    let err = match Client::with_token(test_config(&server.uri()), "bad-token").await {
        Ok(_) => panic!("both endpoints failing must fail with_token"),
        Err(err) => err,
    };
    assert!(err.is_authentication(), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// Auth errors are not retried (Go #29)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_error_190_is_not_retried_even_with_retries_enabled() {
    let server = MockServer::start().await;

    // expect(1) fails the test if the client retries.
    Mock::given(method("GET"))
        .and(path("/debug_token"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {"message": "Invalid OAuth access token", "code": 190}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut config = test_config(&server.uri());
    config.retry_config.max_retries = 3;
    let client = Client::new(config).await.unwrap();
    client
        .set_token_info(TokenInfo {
            access_token: "user-token".into(),
            token_type: "bearer".into(),
            expires_at: Utc::now() + chrono::Duration::days(30),
            user_id: "user-1".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    let err = client.debug_token("user-token").await.unwrap_err();
    assert!(err.is_authentication());
}

// ---------------------------------------------------------------------------
// /threads_publish recovery (Go #30/#31)
// ---------------------------------------------------------------------------

fn published_text_post(id: &str, text: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "media_type": "TEXT_POST",
        "text": text,
        "is_reply": false,
        "is_quote_post": false
    })
}

/// Publish returns HTTP 500 + code 10; container status reports PUBLISHED;
/// the user's recent posts contain exactly one matching post → the client
/// recovers and returns it instead of the error.
#[tokio::test]
async fn create_text_post_recovers_from_code_10_false_failure() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "container-1"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    // expect(1): code 10 must not be retried (it burns publish quota).
    Mock::given(method("POST"))
        .and(path("/user-1/threads_publish"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "message": "Application does not have permission for this action",
                "code": 10
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/container-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "container-1",
            "status": "PUBLISHED"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/user-1/threads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                published_text_post("other-post", "unrelated text"),
                published_text_post("recovered-post", "hello recovery"),
            ]
        })))
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    let post = client
        .create_text_post(&text_content("hello recovery"))
        .await
        .expect("publish should be recovered");
    assert_eq!(post.id.to_string(), "recovered-post");
}

/// Container in a terminal ERROR state means the publish really failed —
/// the original code-10 error must be surfaced, not a recovery result.
#[tokio::test]
async fn create_text_post_surfaces_original_error_when_container_errored() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "container-1"})),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads_publish"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "message": "Application does not have permission for this action",
                "code": 10
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/container-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "container-1",
            "status": "ERROR",
            "error_message": "processing failed"
        })))
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    let err = client
        .create_text_post(&text_content("hello"))
        .await
        .expect_err("terminal container state must surface the publish error");
    let msg = err.to_string();
    assert!(msg.contains("permission"), "unexpected error: {msg}");
}

/// Errors that aren't the known false-failure pattern (code 10) must not
/// trigger recovery at all — no extra Meta round-trips.
#[tokio::test]
async fn create_text_post_does_not_attempt_recovery_for_other_errors() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "container-1"})),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads_publish"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {"message": "Invalid parameter", "code": 100}
        })))
        .mount(&server)
        .await;

    // No container-status mock: a recovery attempt would 404 loudly, but the
    // real assertion is expect(0) below.
    Mock::given(method("GET"))
        .and(path("/container-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "container-1",
            "status": "PUBLISHED"
        })))
        .expect(0)
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    let err = client
        .create_text_post(&text_content("hello"))
        .await
        .expect_err("genuine failure must propagate");
    assert!(err.to_string().contains("Invalid parameter"));
}

/// Multiple posts matching the recovery window is ambiguous — fail closed
/// and surface the original publish error.
#[tokio::test]
async fn recovery_fails_closed_on_ambiguous_match() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "container-1"})),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/user-1/threads_publish"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {"message": "GraphMethodException", "code": 10}
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/container-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "container-1",
            "status": "PUBLISHED"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/user-1/threads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                published_text_post("post-a", "same text"),
                published_text_post("post-b", "same text"),
            ]
        })))
        .mount(&server)
        .await;

    let client = authenticated_client(&server.uri()).await;
    let err = client
        .create_text_post(&text_content("same text"))
        .await
        .expect_err("ambiguous match must fail closed");
    assert!(err.to_string().contains("GraphMethodException"));
}
