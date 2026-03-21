use std::collections::HashMap;

use base64::Engine;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Deserialize;

use crate::client::{Client, TokenInfo};
use crate::error;
use crate::http::RequestBody;

// ---------------------------------------------------------------------------
// OAuth response types
// ---------------------------------------------------------------------------

/// Response from the short-lived token exchange (`/oauth/access_token`).
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    /// The OAuth access token.
    pub access_token: String,
    /// Token type (usually "bearer").
    pub token_type: String,
    /// Token lifetime in seconds.
    pub expires_in: Option<i64>,
    /// App-scoped user ID.
    pub user_id: Option<i64>,
}

/// Response from the long-lived token exchange (`/access_token`).
#[derive(Debug, Deserialize)]
pub struct LongLivedTokenResponse {
    /// The long-lived access token.
    pub access_token: String,
    /// Token type (usually "bearer").
    pub token_type: String,
    /// Token lifetime in seconds (typically 5184000 for 60 days).
    pub expires_in: i64,
}

/// Response from the debug token endpoint (`/debug_token`).
#[derive(Debug, Deserialize)]
pub struct DebugTokenResponse {
    /// Token introspection data.
    pub data: DebugTokenData,
}

/// Inner payload of a debug-token response.
#[derive(Debug, Deserialize)]
pub struct DebugTokenData {
    /// Whether the token is currently valid.
    pub is_valid: bool,
    /// Unix timestamp when the token expires.
    pub expires_at: i64,
    /// Unix timestamp when the token was issued.
    pub issued_at: i64,
    /// OAuth scopes granted to the token.
    pub scopes: Vec<String>,
    /// App-scoped user ID.
    pub user_id: String,
    /// Token type: "USER" or "APP".
    #[serde(default, rename = "type")]
    pub token_type: Option<String>,
    /// Name of the application.
    #[serde(default)]
    pub application: Option<String>,
    /// Unix timestamp when the app's data access expires.
    #[serde(default)]
    pub data_access_expires_at: Option<i64>,
}

/// Response from the app access token endpoint.
#[derive(Debug, Deserialize)]
pub struct AppAccessTokenResponse {
    /// The app access token.
    pub access_token: String,
    /// Token type (usually "bearer").
    pub token_type: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the app access token shorthand string.
///
/// Returns `"TH|{client_id}|{client_secret}"` or an empty string if either is empty.
fn app_access_token_shorthand(client_id: &str, client_secret: &str) -> String {
    if client_id.is_empty() || client_secret.is_empty() {
        return String::new();
    }
    format!("TH|{client_id}|{client_secret}")
}

/// Generate a cryptographically-random state parameter (base64url, 32 bytes).
fn generate_state() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

// ---------------------------------------------------------------------------
// Auth methods on Client
// ---------------------------------------------------------------------------

impl Client {
    /// Build the OAuth authorization URL that the user should visit.
    ///
    /// `scopes` overrides the scopes from the client config. Pass an empty
    /// slice to use the config defaults.
    /// Returns `(url, state)` — the caller must store `state` and verify it
    /// matches the `state` query parameter on the OAuth callback to prevent CSRF.
    pub fn get_auth_url(&self, scopes: &[String]) -> (String, String) {
        let cfg = self.config();
        let effective_scopes = if scopes.is_empty() {
            &cfg.scopes
        } else {
            scopes
        };

        let scope = effective_scopes.join(",");
        let state = generate_state();

        let mut url = url::Url::parse("https://www.threads.net/oauth/authorize")
            .expect("static URL is valid");

        url.query_pairs_mut()
            .append_pair("client_id", &cfg.client_id)
            .append_pair("redirect_uri", &cfg.redirect_uri)
            .append_pair("scope", &scope)
            .append_pair("response_type", "code")
            .append_pair("state", &state);

        (url.into(), state)
    }

    /// Get an app access token using client credentials.
    ///
    /// This does NOT store the token in the client (matches Go behavior).
    /// The caller should use the returned token as needed.
    pub async fn get_app_access_token(&self) -> crate::Result<AppAccessTokenResponse> {
        let cfg = self.config();

        // SECURITY: The Graph API requires client_secret as a query parameter for
        // app access token requests (GET /oauth/access_token). This means the secret
        // appears in server/proxy access logs. Always use HTTPS and ensure log access
        // is restricted.
        let mut params = HashMap::new();
        params.insert("client_id".into(), cfg.client_id.clone());
        params.insert("client_secret".into(), cfg.client_secret.clone());
        params.insert("grant_type".into(), "client_credentials".into());

        let resp = self
            .http_client
            .get("/oauth/access_token", params, "")
            .await?;

        resp.json()
    }

    /// Get an app access token in shorthand format.
    ///
    /// Returns `"TH|{client_id}|{client_secret}"` or an empty string if
    /// `client_id` or `client_secret` are empty.
    pub fn get_app_access_token_shorthand(&self) -> String {
        let cfg = self.config();
        app_access_token_shorthand(&cfg.client_id, &cfg.client_secret)
    }

    /// Exchange an authorization code for a short-lived access token.
    ///
    /// On success the token is stored via `set_token_info`.
    pub async fn exchange_code_for_token(&self, code: &str) -> crate::Result<()> {
        let cfg = self.config().clone();

        let mut form = HashMap::new();
        form.insert("client_id".into(), cfg.client_id);
        form.insert("client_secret".into(), cfg.client_secret);
        form.insert("grant_type".into(), "authorization_code".into());
        form.insert("redirect_uri".into(), cfg.redirect_uri);
        form.insert("code".into(), code.to_owned());

        let resp = self
            .http_client
            .post("/oauth/access_token", Some(RequestBody::Form(form)), "")
            .await?;

        let token_resp: TokenResponse = resp.json()?;

        let expires_in = token_resp.expires_in.unwrap_or(3600);
        let user_id = token_resp
            .user_id
            .map(|id| id.to_string())
            .unwrap_or_default();

        let token_info = TokenInfo {
            access_token: token_resp.access_token,
            token_type: token_resp.token_type,
            expires_at: Utc::now() + chrono::Duration::seconds(expires_in),
            user_id,
            created_at: Utc::now(),
        };

        self.set_token_info(token_info).await
    }

    /// Convert the current short-lived token into a long-lived token (60 days).
    ///
    /// Requires that the client already holds a valid short-lived token.
    pub async fn get_long_lived_token(&self) -> crate::Result<()> {
        let access_token = self.access_token().await;
        if access_token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "No access token available",
                "Call exchange_code_for_token first",
            ));
        }

        let cfg = self.config();

        // SECURITY: The Graph API requires client_secret as a query parameter for
        // long-lived token exchange (GET /access_token). This means the secret appears
        // in server/proxy access logs. Always use HTTPS and ensure log access is restricted.
        let mut params = HashMap::new();
        params.insert("grant_type".into(), "th_exchange_token".into());
        params.insert("client_secret".into(), cfg.client_secret.clone());
        params.insert("access_token".into(), access_token.clone());

        let resp = self
            .http_client
            .get("/access_token", params, &access_token)
            .await?;

        let long_resp: LongLivedTokenResponse = resp.json()?;

        let user_id = self.user_id().await;

        let token_info = TokenInfo {
            access_token: long_resp.access_token,
            token_type: long_resp.token_type,
            expires_at: Utc::now() + chrono::Duration::seconds(long_resp.expires_in),
            user_id,
            created_at: Utc::now(),
        };

        self.set_token_info(token_info).await
    }

    /// Refresh the current long-lived token, extending its expiry.
    ///
    /// The token must still be valid (not expired) to be refreshed.
    pub async fn refresh_token(&self) -> crate::Result<()> {
        let access_token = self.access_token().await;
        if access_token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "No access token available",
                "Cannot refresh without a valid token",
            ));
        }

        let mut params = HashMap::new();
        params.insert("grant_type".into(), "th_refresh_token".into());
        params.insert("access_token".into(), access_token.clone());

        let resp = self
            .http_client
            .get("/refresh_access_token", params, &access_token)
            .await?;

        let long_resp: LongLivedTokenResponse = resp.json()?;

        let user_id = self.user_id().await;

        let token_info = TokenInfo {
            access_token: long_resp.access_token,
            token_type: long_resp.token_type,
            expires_at: Utc::now() + chrono::Duration::seconds(long_resp.expires_in),
            user_id,
            created_at: Utc::now(),
        };

        self.set_token_info(token_info).await
    }

    /// Inspect a token via the `/debug_token` endpoint.
    pub async fn debug_token(&self, input_token: &str) -> crate::Result<DebugTokenResponse> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(crate::error::new_authentication_error(
                401,
                "Access token is required to call debug_token",
                "",
            ));
        }

        let mut params = HashMap::new();
        params.insert("input_token".into(), input_token.to_owned());

        let resp = self.http_client.get("/debug_token", params, &token).await?;

        resp.json()
    }

    /// Validate the current token locally: non-empty and not expired.
    pub async fn validate_token(&self) -> crate::Result<()> {
        let state = self.get_token_info().await;
        match state {
            Some(info) => {
                if info.access_token.is_empty() {
                    return Err(error::new_authentication_error(401, "Token is empty", ""));
                }
                if Utc::now() > info.expires_at {
                    return Err(error::new_authentication_error(
                        401,
                        "Token has expired",
                        "",
                    ));
                }
                Ok(())
            }
            None => Err(error::new_authentication_error(
                401,
                "No token available",
                "",
            )),
        }
    }

    /// Validate the current token and auto-refresh if expired.
    ///
    /// Only attempts a refresh when the token exists but has expired.
    /// Returns the original error for other failures (no token, empty token).
    pub async fn ensure_valid_token(&self) -> crate::Result<()> {
        match self.validate_token().await {
            Ok(()) => Ok(()),
            Err(e) => {
                // Only refresh if we have a token that expired
                if self.is_token_expired().await && self.get_token_info().await.is_some() {
                    self.refresh_token().await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Return debug information about the current token.
    ///
    /// The access token is masked (first 4 + last 4 characters shown).
    pub async fn get_token_debug_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        let state = self.get_token_info().await;
        match state {
            Some(token_info) => {
                let masked = if token_info.access_token.len() > 8 {
                    let len = token_info.access_token.len();
                    format!(
                        "{}...{}",
                        &token_info.access_token[..4],
                        &token_info.access_token[len - 4..]
                    )
                } else {
                    "****".to_owned()
                };
                info.insert("access_token".into(), masked);
                info.insert("token_type".into(), token_info.token_type.clone());
                info.insert("expires_at".into(), token_info.expires_at.to_rfc3339());
                info.insert("user_id".into(), token_info.user_id.clone());
                info.insert("created_at".into(), token_info.created_at.to_rfc3339());
                info.insert(
                    "is_expired".into(),
                    (Utc::now() > token_info.expires_at).to_string(),
                );
            }
            None => {
                info.insert("status".into(), "no_token".into());
            }
        }
        info
    }

    /// Explicitly reload the token from storage.
    pub async fn load_token_from_storage(&self) -> crate::Result<()> {
        let loaded = self.token_storage.load().await?;
        self.set_token_info(loaded).await
    }

    /// Store a token built from a previous `debug_token` response.
    ///
    /// Useful for bootstrapping the client from a known-valid token without
    /// going through the full OAuth flow again.
    pub async fn set_token_from_debug_info(
        &self,
        access_token: &str,
        debug_resp: &DebugTokenResponse,
    ) -> crate::Result<()> {
        let data = &debug_resp.data;

        if !data.is_valid {
            return Err(error::new_authentication_error(
                401,
                "Cannot set token from invalid debug info: token is not valid",
                "",
            ));
        }

        let expires_at =
            DateTime::<Utc>::from_timestamp(data.expires_at, 0).unwrap_or_else(Utc::now);

        let created_at =
            DateTime::<Utc>::from_timestamp(data.issued_at, 0).unwrap_or_else(Utc::now);

        let token_info = TokenInfo {
            access_token: access_token.to_owned(),
            token_type: "bearer".into(),
            expires_at,
            user_id: data.user_id.clone(),
            created_at,
        };

        self.set_token_info(token_info).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Config;

    fn test_config() -> Config {
        Config::new(
            "test-client-id",
            "test-secret",
            "https://example.com/callback",
        )
    }

    #[test]
    fn test_generate_state_unique() {
        let a = generate_state();
        let b = generate_state();
        assert_ne!(a, b);
        // base64url of 32 bytes = 43 chars (no padding)
        assert_eq!(a.len(), 43);
    }

    #[test]
    fn test_generate_state_is_valid_base64url() {
        let s = generate_state();
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&s)
            .expect("should be valid base64url");
        assert_eq!(decoded.len(), 32);
    }

    #[tokio::test]
    async fn test_get_auth_url_contains_required_params() {
        let client = Client::new(test_config()).await.unwrap();
        let (url, state) = client.get_auth_url(&[]);

        assert!(url.starts_with("https://www.threads.net/oauth/authorize?"));
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state="));
        assert!(url.contains("scope="));
        assert!(
            !state.is_empty(),
            "state must be returned for CSRF verification"
        );
        assert!(url.contains(&format!("state={state}")));
    }

    #[tokio::test]
    async fn test_get_auth_url_uses_custom_scopes() {
        let client = Client::new(test_config()).await.unwrap();
        let scopes = vec!["threads_basic".into(), "threads_manage_replies".into()];
        let (url, _state) = client.get_auth_url(&scopes);

        // comma-joined in the scope param
        assert!(url.contains("scope=threads_basic%2Cthreads_manage_replies"));
    }

    #[tokio::test]
    async fn test_get_auth_url_uses_config_scopes_when_empty() {
        let client = Client::new(test_config()).await.unwrap();
        let (url, _state) = client.get_auth_url(&[]);

        // Config default includes threads_basic
        assert!(url.contains("threads_basic"));
    }

    #[test]
    fn test_token_response_deserialize() {
        let json = r#"{
            "access_token": "tok_abc",
            "token_type": "bearer",
            "expires_in": 3600,
            "user_id": 12345
        }"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "tok_abc");
        assert_eq!(resp.token_type, "bearer");
        assert_eq!(resp.expires_in, Some(3600));
        assert_eq!(resp.user_id, Some(12345));
    }

    #[test]
    fn test_token_response_deserialize_optional_fields() {
        let json = r#"{
            "access_token": "tok_abc",
            "token_type": "bearer"
        }"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(resp.expires_in.is_none());
        assert!(resp.user_id.is_none());
    }

    #[test]
    fn test_long_lived_token_response_deserialize() {
        let json = r#"{
            "access_token": "long_tok",
            "token_type": "bearer",
            "expires_in": 5184000
        }"#;
        let resp: LongLivedTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "long_tok");
        assert_eq!(resp.expires_in, 5184000);
    }

    #[test]
    fn test_debug_token_response_deserialize() {
        let json = r#"{
            "data": {
                "is_valid": true,
                "expires_at": 1700000000,
                "issued_at": 1699900000,
                "scopes": ["threads_basic", "threads_content_publish"],
                "user_id": "987654"
            }
        }"#;
        let resp: DebugTokenResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_valid);
        assert_eq!(resp.data.expires_at, 1700000000);
        assert_eq!(resp.data.issued_at, 1699900000);
        assert_eq!(resp.data.scopes.len(), 2);
        assert_eq!(resp.data.user_id, "987654");
    }

    #[tokio::test]
    async fn test_validate_token_no_token() {
        let client = Client::new(test_config()).await.unwrap();
        assert!(client.validate_token().await.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_valid() {
        let client = Client::new(test_config()).await.unwrap();
        let token = crate::client::TokenInfo {
            access_token: "valid-tok".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "u-1".into(),
            created_at: Utc::now(),
        };
        client.set_token_info(token).await.unwrap();
        assert!(client.validate_token().await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_token_expired() {
        let client = Client::new(test_config()).await.unwrap();
        let token = crate::client::TokenInfo {
            access_token: "expired-tok".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            user_id: "u-1".into(),
            created_at: Utc::now() - chrono::Duration::hours(2),
        };
        client.set_token_info(token).await.unwrap();
        assert!(client.validate_token().await.is_err());
    }

    #[tokio::test]
    async fn test_get_token_debug_info_no_token() {
        let client = Client::new(test_config()).await.unwrap();
        let info = client.get_token_debug_info().await;
        assert_eq!(info.get("status").unwrap(), "no_token");
    }

    #[tokio::test]
    async fn test_get_token_debug_info_with_token() {
        let client = Client::new(test_config()).await.unwrap();
        let token = crate::client::TokenInfo {
            access_token: "abcdefghijklmnop".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "u-1".into(),
            created_at: Utc::now(),
        };
        client.set_token_info(token).await.unwrap();
        let info = client.get_token_debug_info().await;
        let masked = info.get("access_token").unwrap();
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("mnop"));
        assert!(masked.contains("..."));
        assert_eq!(info.get("user_id").unwrap(), "u-1");
        assert_eq!(info.get("is_expired").unwrap(), "false");
    }

    #[tokio::test]
    async fn test_load_token_from_storage_empty() {
        let client = Client::new(test_config()).await.unwrap();
        // No token stored — should error
        assert!(client.load_token_from_storage().await.is_err());
    }

    #[test]
    fn test_app_access_token_response_deserialize() {
        let json = r#"{
            "access_token": "app_tok_abc",
            "token_type": "bearer"
        }"#;
        let resp: AppAccessTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "app_tok_abc");
        assert_eq!(resp.token_type, "bearer");
    }

    #[tokio::test]
    async fn test_get_app_access_token_shorthand() {
        let client = Client::new(test_config()).await.unwrap();
        let shorthand = client.get_app_access_token_shorthand();
        assert_eq!(shorthand, "TH|test-client-id|test-secret");
    }

    #[test]
    fn test_app_access_token_shorthand_empty_client_id() {
        assert_eq!(app_access_token_shorthand("", "secret"), "");
    }

    #[test]
    fn test_app_access_token_shorthand_empty_secret() {
        assert_eq!(app_access_token_shorthand("id", ""), "");
    }

    #[test]
    fn test_app_access_token_shorthand_both_empty() {
        assert_eq!(app_access_token_shorthand("", ""), "");
    }
}
