//! Integration tests for the `token_type` normalization ported from threads-go
//! (#39): every path that produces a `TokenInfo` — the token endpoints,
//! persisted storage written by older versions, and callers building
//! `TokenInfo` by hand — yields the canonical `Bearer` spelling.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use threads_rs::client::{Client, Config, TokenInfo, TokenStorage};
use threads_rs::constants::TOKEN_TYPE_BEARER;

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

fn stored_token(token_type: &str) -> TokenInfo {
    TokenInfo {
        access_token: "persisted".into(),
        token_type: token_type.to_owned(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        user_id: "12345".into(),
        created_at: Utc::now(),
    }
}

/// Token storage pre-seeded with a token written by an older version. The
/// contents are shared with the test so it can read what was persisted
/// without going back through the client.
struct SeededStorage(Arc<Mutex<TokenInfo>>);

impl SeededStorage {
    fn seeded(token_type: &str) -> (Box<Self>, Arc<Mutex<TokenInfo>>) {
        let cell = Arc::new(Mutex::new(stored_token(token_type)));
        (Box::new(Self(Arc::clone(&cell))), cell)
    }
}

impl TokenStorage for SeededStorage {
    fn store(
        &self,
        token: &TokenInfo,
    ) -> Pin<Box<dyn Future<Output = threads_rs::Result<()>> + Send + '_>> {
        let token = token.clone();
        Box::pin(async move {
            *self.0.lock().unwrap() = token;
            Ok(())
        })
    }

    fn load(&self) -> Pin<Box<dyn Future<Output = threads_rs::Result<TokenInfo>> + Send + '_>> {
        Box::pin(async move { Ok(self.0.lock().unwrap().clone()) })
    }

    fn delete(&self) -> Pin<Box<dyn Future<Output = threads_rs::Result<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
}

async fn mock_server(endpoint: &str, body: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_string(body.to_owned()))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn exchange_code_for_token_normalizes_token_type() {
    let server = mock_server(
        "/oauth/access_token",
        r#"{"access_token":"tok","token_type":"bearer","expires_in":3600,"user_id":123}"#,
    )
    .await;

    let client = Client::new(test_config(&server.uri())).await.unwrap();
    client
        .exchange_code_for_token("code", "state", "state")
        .await
        .unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.token_type, TOKEN_TYPE_BEARER);
}

#[tokio::test]
async fn exchange_code_for_token_defaults_missing_token_type() {
    let server = mock_server(
        "/oauth/access_token",
        r#"{"access_token":"tok","expires_in":3600,"user_id":123}"#,
    )
    .await;

    let client = Client::new(test_config(&server.uri())).await.unwrap();
    client
        .exchange_code_for_token("code", "state", "state")
        .await
        .unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.token_type, TOKEN_TYPE_BEARER);
}

#[tokio::test]
async fn get_long_lived_token_normalizes_token_type() {
    let server = mock_server(
        "/access_token",
        r#"{"access_token":"ll","token_type":"bearer","expires_in":5184000}"#,
    )
    .await;

    let client = Client::new(test_config(&server.uri())).await.unwrap();
    client.set_token_info(stored_token("bearer")).await.unwrap();
    client.get_long_lived_token().await.unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.access_token, "ll");
    assert_eq!(info.token_type, TOKEN_TYPE_BEARER);
}

#[tokio::test]
async fn refresh_token_normalizes_token_type() {
    let server = mock_server(
        "/refresh_access_token",
        r#"{"access_token":"ref","token_type":"bearer","expires_in":5184000}"#,
    )
    .await;

    let client = Client::new(test_config(&server.uri())).await.unwrap();
    client.set_token_info(stored_token("bearer")).await.unwrap();
    client.refresh_token().await.unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.access_token, "ref");
    assert_eq!(info.token_type, TOKEN_TYPE_BEARER);
}

#[tokio::test]
async fn client_construction_normalizes_token_type_from_storage() {
    for stored in ["", "bearer"] {
        let (storage, _) = SeededStorage::seeded(stored);
        let client = Client::with_token_storage(test_config("https://example.com"), storage)
            .await
            .unwrap();

        let info = client.get_token_info().await.unwrap();
        assert_eq!(info.token_type, TOKEN_TYPE_BEARER, "stored: {stored:?}");
        assert_eq!(
            client.get_token_debug_info().await.get("token_type"),
            Some(&TOKEN_TYPE_BEARER.to_owned()),
            "stored: {stored:?}"
        );
    }
}

#[tokio::test]
async fn load_token_from_storage_normalizes_token_type() {
    for stored in ["", "bearer"] {
        let (storage, _) = SeededStorage::seeded(stored);
        let client = Client::with_token_storage(test_config("https://example.com"), storage)
            .await
            .unwrap();

        client.load_token_from_storage().await.unwrap();

        let info = client.get_token_info().await.unwrap();
        assert_eq!(info.token_type, TOKEN_TYPE_BEARER, "stored: {stored:?}");
    }
}

#[tokio::test]
async fn set_token_info_normalizes_hand_built_tokens_and_persists_them() {
    let (storage, persisted) = SeededStorage::seeded("bearer");
    let client = Client::with_token_storage(test_config("https://example.com"), storage)
        .await
        .unwrap();

    client.set_token_info(stored_token("")).await.unwrap();

    let info = client.get_token_info().await.unwrap();
    assert_eq!(info.token_type, TOKEN_TYPE_BEARER);

    // The normalized value is what reaches storage, not the raw one — read the
    // store directly, since loading back through the client would normalize
    // again and hide a raw write.
    assert_eq!(persisted.lock().unwrap().token_type, TOKEN_TYPE_BEARER);

    // A non-bearer scheme is preserved verbatim, in memory and in storage.
    client.set_token_info(stored_token("mac")).await.unwrap();
    assert_eq!(client.get_token_info().await.unwrap().token_type, "mac");
    assert_eq!(persisted.lock().unwrap().token_type, "mac");
}
