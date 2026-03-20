use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::constants::{BASE_API_URL, DEFAULT_HTTP_TIMEOUT, VERSION};
use crate::error;
use crate::http::{HttpClient, RetryConfig};
use crate::rate_limit::{RateLimitStatus, RateLimiter, RateLimiterConfig};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Valid OAuth scopes for the Threads API.
pub const VALID_SCOPES: &[&str] = &[
    "threads_basic",
    "threads_content_publish",
    "threads_manage_insights",
    "threads_manage_replies",
    "threads_read_replies",
    "threads_manage_mentions",
    "threads_keyword_search",
    "threads_delete",
    "threads_location_tagging",
    "threads_profile_discovery",
];

/// Configuration for the Threads API client.
#[derive(Debug, Clone)]
pub struct Config {
    /// OAuth application client ID.
    pub client_id: String,
    /// OAuth application client secret.
    pub client_secret: String,
    /// OAuth redirect URI.
    pub redirect_uri: String,
    /// OAuth scopes to request.
    pub scopes: Vec<String>,
    /// HTTP request timeout.
    pub http_timeout: Duration,
    /// Retry configuration for failed requests.
    pub retry_config: RetryConfig,
    /// Base URL for the Threads API.
    pub base_url: String,
    /// User-Agent header value.
    pub user_agent: String,
    /// Enable debug logging.
    pub debug: bool,
}

impl Config {
    /// Create a new config with required fields and sensible defaults.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            redirect_uri: redirect_uri.into(),
            scopes: vec![
                "threads_basic".into(),
                "threads_content_publish".into(),
                "threads_manage_replies".into(),
                "threads_manage_insights".into(),
                "threads_read_replies".into(),
                "threads_manage_mentions".into(),
                "threads_keyword_search".into(),
                "threads_delete".into(),
                "threads_location_tagging".into(),
                "threads_profile_discovery".into(),
            ],
            http_timeout: DEFAULT_HTTP_TIMEOUT,
            retry_config: RetryConfig::default(),
            base_url: BASE_API_URL.to_owned(),
            user_agent: format!("threads-api-rust/{}", VERSION),
            debug: false,
        }
    }

    /// Create a config from environment variables.
    ///
    /// Required: `THREADS_CLIENT_ID`, `THREADS_CLIENT_SECRET`, `THREADS_REDIRECT_URI`.
    pub fn from_env() -> crate::Result<Self> {
        let client_id = std::env::var("THREADS_CLIENT_ID").map_err(|_| {
            error::new_validation_error(
                0,
                "THREADS_CLIENT_ID environment variable is required",
                "",
                "client_id",
            )
        })?;

        let client_secret = std::env::var("THREADS_CLIENT_SECRET").map_err(|_| {
            error::new_validation_error(
                0,
                "THREADS_CLIENT_SECRET environment variable is required",
                "",
                "client_secret",
            )
        })?;

        let redirect_uri = std::env::var("THREADS_REDIRECT_URI").map_err(|_| {
            error::new_validation_error(
                0,
                "THREADS_REDIRECT_URI environment variable is required",
                "",
                "redirect_uri",
            )
        })?;

        let mut config = Self::new(client_id, client_secret, redirect_uri);

        if let Ok(scopes) = std::env::var("THREADS_SCOPES") {
            config.scopes = scopes.split(',').map(|s| s.trim().to_owned()).collect();
        }

        if let Ok(timeout) = std::env::var("THREADS_HTTP_TIMEOUT") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.http_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(base_url) = std::env::var("THREADS_BASE_URL") {
            config.base_url = base_url;
        }

        if let Ok(ua) = std::env::var("THREADS_USER_AGENT") {
            config.user_agent = ua;
        }

        if let Ok(debug) = std::env::var("THREADS_DEBUG") {
            config.debug = debug.parse().unwrap_or(false);
        }

        if let Ok(retries) = std::env::var("THREADS_MAX_RETRIES") {
            if let Ok(n) = retries.parse::<u32>() {
                config.retry_config.max_retries = n;
            }
        }

        Ok(config)
    }

    /// Set defaults for any unset/zero values.
    pub fn set_defaults(&mut self) {
        if self.scopes.is_empty() {
            self.scopes = vec![
                "threads_basic".into(),
                "threads_content_publish".into(),
                "threads_manage_insights".into(),
                "threads_manage_replies".into(),
                "threads_read_replies".into(),
            ];
        }
        if self.http_timeout.is_zero() {
            self.http_timeout = DEFAULT_HTTP_TIMEOUT;
        }
        if self.base_url.is_empty() {
            self.base_url = BASE_API_URL.to_owned();
        }
        if self.user_agent.is_empty() {
            self.user_agent = format!("threads-api-rust/{}", VERSION);
        }
    }

    /// Validate configuration and return an error if invalid.
    pub fn validate(&self) -> crate::Result<()> {
        if self.client_id.is_empty() {
            return Err(error::new_validation_error(
                0,
                "ClientID is required",
                "",
                "client_id",
            ));
        }
        if self.client_secret.is_empty() {
            return Err(error::new_validation_error(
                0,
                "ClientSecret is required",
                "",
                "client_secret",
            ));
        }
        if self.redirect_uri.is_empty() {
            return Err(error::new_validation_error(
                0,
                "RedirectURI is required",
                "",
                "redirect_uri",
            ));
        }
        if !self.redirect_uri.starts_with("http://") && !self.redirect_uri.starts_with("https://") {
            return Err(error::new_validation_error(
                0,
                "RedirectURI must be a valid HTTP or HTTPS URL",
                "",
                "redirect_uri",
            ));
        }
        if self.scopes.is_empty() {
            return Err(error::new_validation_error(
                0,
                "At least one scope is required",
                "",
                "scopes",
            ));
        }
        for scope in &self.scopes {
            if !VALID_SCOPES.contains(&scope.as_str()) {
                return Err(error::new_validation_error(
                    0,
                    &format!("Invalid scope: {}", scope),
                    "",
                    "scopes",
                ));
            }
        }
        if self.http_timeout.is_zero() {
            return Err(error::new_validation_error(
                0,
                "HTTPTimeout must be positive",
                "",
                "http_timeout",
            ));
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(error::new_validation_error(
                0,
                "BaseURL must be a valid HTTP or HTTPS URL",
                "",
                "base_url",
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

/// Information about the current access token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenInfo {
    /// The OAuth access token.
    pub access_token: String,
    /// Token type (usually "bearer").
    pub token_type: String,
    /// When the token expires.
    pub expires_at: DateTime<Utc>,
    /// App-scoped user ID.
    pub user_id: String,
    /// When the token was created.
    pub created_at: DateTime<Utc>,
}

/// Trait for persistent token storage.
///
/// All methods are async so implementations can perform I/O (file, database,
/// remote store) without blocking the Tokio executor.
pub trait TokenStorage: Send + Sync {
    /// Store a token.
    fn store(
        &self,
        token: &TokenInfo,
    ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>>;
    /// Load the stored token.
    fn load(&self) -> Pin<Box<dyn Future<Output = crate::Result<TokenInfo>> + Send + '_>>;
    /// Delete the stored token.
    fn delete(&self) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>>;
}

/// In-memory token storage (default).
pub struct MemoryTokenStorage {
    token: std::sync::Mutex<Option<TokenInfo>>,
}

impl MemoryTokenStorage {
    /// Create a new empty in-memory token store.
    pub fn new() -> Self {
        Self {
            token: std::sync::Mutex::new(None),
        }
    }
}

impl Default for MemoryTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStorage for MemoryTokenStorage {
    fn store(
        &self,
        token: &TokenInfo,
    ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>> {
        let token = token.clone();
        Box::pin(async move {
            let mut guard = self.token.lock().map_err(|_| {
                error::new_authentication_error(500, "Token storage lock poisoned", "")
            })?;
            *guard = Some(token);
            Ok(())
        })
    }

    fn load(&self) -> Pin<Box<dyn Future<Output = crate::Result<TokenInfo>> + Send + '_>> {
        Box::pin(async move {
            let guard = self.token.lock().map_err(|_| {
                error::new_authentication_error(500, "Token storage lock poisoned", "")
            })?;
            guard.clone().ok_or_else(|| {
                error::new_authentication_error(
                    401,
                    "No token stored",
                    "Token not found in memory storage",
                )
            })
        })
    }

    fn delete(&self) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut guard = self.token.lock().map_err(|_| {
                error::new_authentication_error(500, "Token storage lock poisoned", "")
            })?;
            *guard = None;
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Thread-safe state behind RwLock.
struct TokenState {
    access_token: String,
    token_info: Option<TokenInfo>,
}

/// Threads API client. Thread-safe; wrap in `Arc<Client>` to share across tasks.
pub struct Client {
    config: Config,
    pub(crate) http_client: HttpClient,
    rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) token_storage: Box<dyn TokenStorage>,
    token_state: RwLock<TokenState>,
}

impl Client {
    /// Create a new client with the given configuration.
    pub async fn new(mut config: Config) -> crate::Result<Self> {
        config.set_defaults();
        config.validate()?;

        let rate_limiter = Arc::new(RateLimiter::new(&RateLimiterConfig::default()));

        let http_client = HttpClient::new(
            config.http_timeout,
            config.retry_config.clone(),
            Some(Arc::clone(&rate_limiter)),
            Some(&config.base_url),
            Some(&config.user_agent),
        )?;

        let token_storage: Box<dyn TokenStorage> = Box::new(MemoryTokenStorage::new());

        // Try to load existing token
        let (access_token, token_info) = if let Ok(info) = token_storage.load().await {
            let at = info.access_token.clone();
            (at, Some(info))
        } else {
            (String::new(), None)
        };

        Ok(Self {
            config,
            http_client,
            rate_limiter: Some(rate_limiter),
            token_storage,
            token_state: RwLock::new(TokenState {
                access_token,
                token_info,
            }),
        })
    }

    /// Create a new client with custom token storage.
    pub async fn with_token_storage(
        mut config: Config,
        token_storage: Box<dyn TokenStorage>,
    ) -> crate::Result<Self> {
        config.set_defaults();
        config.validate()?;

        let rate_limiter = Arc::new(RateLimiter::new(&RateLimiterConfig::default()));

        let http_client = HttpClient::new(
            config.http_timeout,
            config.retry_config.clone(),
            Some(Arc::clone(&rate_limiter)),
            Some(&config.base_url),
            Some(&config.user_agent),
        )?;

        let (access_token, token_info) = if let Ok(info) = token_storage.load().await {
            let at = info.access_token.clone();
            (at, Some(info))
        } else {
            (String::new(), None)
        };

        Ok(Self {
            config,
            http_client,
            rate_limiter: Some(rate_limiter),
            token_storage,
            token_state: RwLock::new(TokenState {
                access_token,
                token_info,
            }),
        })
    }

    /// Create a client with a pre-existing access token.
    ///
    /// Calls the debug_token endpoint to resolve the user ID and exact
    /// expiry from the token. Useful for scripts and tests where the
    /// token is already known.
    pub async fn with_token(mut config: Config, access_token: &str) -> crate::Result<Self> {
        config.set_defaults();
        config.validate()?;

        let rate_limiter = Arc::new(RateLimiter::new(&RateLimiterConfig::default()));

        let http_client = HttpClient::new(
            config.http_timeout,
            config.retry_config.clone(),
            Some(Arc::clone(&rate_limiter)),
            Some(&config.base_url),
            Some(&config.user_agent),
        )?;

        let token_storage: Box<dyn TokenStorage> = Box::new(MemoryTokenStorage::new());

        // Set a temporary token so we can call debug_token
        let temp_info = TokenInfo {
            access_token: access_token.to_owned(),
            token_type: "bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: String::new(),
            created_at: Utc::now(),
        };

        let client = Self {
            config,
            http_client,
            rate_limiter: Some(rate_limiter),
            token_storage,
            token_state: RwLock::new(TokenState {
                access_token: access_token.to_owned(),
                token_info: Some(temp_info),
            }),
        };

        // Validate and resolve accurate token info via debug_token
        let debug_resp = client.debug_token(access_token).await?;
        client
            .set_token_from_debug_info(access_token, &debug_resp)
            .await?;

        Ok(client)
    }

    /// Create a client from environment variables.
    pub async fn from_env() -> crate::Result<Self> {
        let config = Config::from_env()?;
        Self::new(config).await
    }

    // ---- Token management ----

    /// Set token information (thread-safe).
    pub async fn set_token_info(&self, token_info: TokenInfo) -> crate::Result<()> {
        self.token_storage.store(&token_info).await?;
        let mut state = self.token_state.write().await;
        state.access_token = token_info.access_token.clone();
        state.token_info = Some(token_info);
        Ok(())
    }

    /// Get a copy of the current token info.
    pub async fn get_token_info(&self) -> Option<TokenInfo> {
        self.token_state.read().await.token_info.clone()
    }

    /// Returns `true` if the client has a valid access token.
    pub async fn is_authenticated(&self) -> bool {
        let state = self.token_state.read().await;
        !state.access_token.is_empty() && state.token_info.is_some()
    }

    /// Returns `true` if the current token has expired.
    pub async fn is_token_expired(&self) -> bool {
        let state = self.token_state.read().await;
        match &state.token_info {
            Some(info) => Utc::now() > info.expires_at,
            None => true,
        }
    }

    /// Returns `true` if the token expires within the given duration.
    pub async fn is_token_expiring_soon(&self, within: Duration) -> bool {
        let state = self.token_state.read().await;
        match &state.token_info {
            Some(info) => {
                let threshold = Utc::now()
                    + chrono::Duration::from_std(within).unwrap_or(chrono::Duration::zero());
                threshold > info.expires_at
            }
            None => true,
        }
    }

    /// Clear the current token from the client and storage.
    pub async fn clear_token(&self) -> crate::Result<()> {
        self.token_storage.delete().await?;
        let mut state = self.token_state.write().await;
        state.access_token.clear();
        state.token_info = None;
        Ok(())
    }

    /// Get the current access token.
    pub async fn access_token(&self) -> String {
        self.token_state.read().await.access_token.clone()
    }

    /// Get the user ID from the current token info.
    pub(crate) async fn user_id(&self) -> String {
        self.token_state
            .read()
            .await
            .token_info
            .as_ref()
            .map(|t| t.user_id.clone())
            .unwrap_or_default()
    }

    // ---- Config access ----

    /// Returns a reference to the client configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Consume this client and create a new one with the given config,
    /// preserving the current token state.
    pub async fn update_config(self, mut new_config: Config) -> crate::Result<Client> {
        new_config.set_defaults();
        new_config.validate()?;

        let rate_limiter = Arc::new(RateLimiter::new(&RateLimiterConfig::default()));

        let http_client = HttpClient::new(
            new_config.http_timeout,
            new_config.retry_config.clone(),
            Some(Arc::clone(&rate_limiter)),
            Some(&new_config.base_url),
            Some(&new_config.user_agent),
        )?;

        let state = self.token_state.read().await;
        let access_token = state.access_token.clone();
        let token_info = state.token_info.clone();
        drop(state);

        Ok(Client {
            config: new_config,
            http_client,
            rate_limiter: Some(rate_limiter),
            token_storage: self.token_storage,
            token_state: RwLock::new(TokenState {
                access_token,
                token_info,
            }),
        })
    }

    // ---- Rate limit access ----

    /// Returns the current rate limit status.
    pub async fn rate_limit_status(&self) -> Option<RateLimitStatus> {
        if let Some(ref rl) = self.rate_limiter {
            Some(rl.get_status().await)
        } else {
            None
        }
    }

    /// Returns `true` if we're close to the rate limit.
    pub async fn is_near_rate_limit(&self, threshold: f64) -> bool {
        if let Some(ref rl) = self.rate_limiter {
            rl.is_near_limit(threshold).await
        } else {
            false
        }
    }

    /// Returns `true` if we're currently rate-limited by the API.
    pub async fn is_rate_limited(&self) -> bool {
        if let Some(ref rl) = self.rate_limiter {
            rl.is_rate_limited().await
        } else {
            false
        }
    }

    /// Disable rate limiting. Requests will not be throttled.
    pub async fn disable_rate_limiting(&self) {
        if let Some(ref rl) = self.rate_limiter {
            rl.disable().await;
        }
    }

    /// Enable rate limiting.
    pub async fn enable_rate_limiting(&self) {
        if let Some(ref rl) = self.rate_limiter {
            rl.enable().await;
        }
    }

    /// Wait until the rate limiter allows a request.
    pub async fn wait_for_rate_limit(&self) -> crate::Result<()> {
        if let Some(ref rl) = self.rate_limiter {
            if rl.should_wait().await {
                rl.wait().await?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::new(
            "test-client-id",
            "test-secret",
            "https://example.com/callback",
        )
    }

    #[test]
    fn test_config_new_defaults() {
        let cfg = test_config();
        assert_eq!(cfg.client_id, "test-client-id");
        assert_eq!(cfg.base_url, BASE_API_URL);
        assert_eq!(cfg.http_timeout, DEFAULT_HTTP_TIMEOUT);
        assert!(!cfg.scopes.is_empty());
    }

    #[test]
    fn test_config_validate_ok() {
        let cfg = test_config();
        cfg.validate().unwrap();
    }

    #[test]
    fn test_config_validate_empty_client_id() {
        let cfg = Config::new("", "secret", "https://example.com/cb");
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_validate_bad_redirect_uri() {
        let cfg = Config::new("id", "secret", "not-a-url");
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_scope() {
        let mut cfg = test_config();
        cfg.scopes.push("invalid_scope".into());
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_scopes() {
        let mut cfg = test_config();
        cfg.scopes.clear();
        assert!(cfg.validate().is_err());
    }

    #[tokio::test]
    async fn test_memory_token_storage() {
        let storage = MemoryTokenStorage::new();
        assert!(storage.load().await.is_err());

        let token = TokenInfo {
            access_token: "test-token".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "user-1".into(),
            created_at: Utc::now(),
        };

        storage.store(&token).await.unwrap();
        let loaded = storage.load().await.unwrap();
        assert_eq!(loaded.access_token, "test-token");

        storage.delete().await.unwrap();
        assert!(storage.load().await.is_err());
    }

    #[tokio::test]
    async fn test_client_new() {
        let client = Client::new(test_config()).await.unwrap();
        assert!(!client.is_authenticated().await);
        assert!(client.is_token_expired().await);
    }

    #[tokio::test]
    async fn test_client_set_and_get_token() {
        let client = Client::new(test_config()).await.unwrap();
        let token = TokenInfo {
            access_token: "my-token".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "u-123".into(),
            created_at: Utc::now(),
        };

        client.set_token_info(token).await.unwrap();
        assert!(client.is_authenticated().await);
        assert!(!client.is_token_expired().await);
        assert_eq!(client.access_token().await, "my-token");
        assert_eq!(client.user_id().await, "u-123");
    }

    #[tokio::test]
    async fn test_client_clear_token() {
        let client = Client::new(test_config()).await.unwrap();
        let token = TokenInfo {
            access_token: "tok".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "u-1".into(),
            created_at: Utc::now(),
        };

        client.set_token_info(token).await.unwrap();
        assert!(client.is_authenticated().await);

        client.clear_token().await.unwrap();
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_client_token_expiring_soon() {
        let client = Client::new(test_config()).await.unwrap();
        let token = TokenInfo {
            access_token: "tok".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
            user_id: "u-1".into(),
            created_at: Utc::now(),
        };

        client.set_token_info(token).await.unwrap();
        assert!(
            client
                .is_token_expiring_soon(Duration::from_secs(3600))
                .await
        );
        assert!(!client.is_token_expiring_soon(Duration::from_secs(60)).await);
    }

    #[tokio::test]
    async fn test_client_rate_limit_status() {
        let client = Client::new(test_config()).await.unwrap();
        let status = client.rate_limit_status().await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().limit, 100);
    }

    // with_token requires a live API call (debug_token), so it is tested
    // in examples/validate.rs rather than here.

    #[tokio::test]
    async fn test_client_update_config() {
        let client = Client::new(test_config()).await.unwrap();
        let token = TokenInfo {
            access_token: "keep-me".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_id: "u-1".into(),
            created_at: Utc::now(),
        };
        client.set_token_info(token).await.unwrap();

        let mut new_config = test_config();
        new_config.debug = true;
        let new_client = client.update_config(new_config).await.unwrap();

        assert!(new_client.config().debug);
        assert_eq!(new_client.access_token().await, "keep-me");
    }

    #[tokio::test]
    async fn test_client_disable_enable_rate_limiting() {
        let client = Client::new(test_config()).await.unwrap();
        assert!(!client.is_rate_limited().await);

        client.disable_rate_limiting().await;
        // Even if marked rate limited, should not report as limited when disabled
        // (tested at the rate_limiter level)

        client.enable_rate_limiting().await;
    }

    #[tokio::test]
    async fn test_client_wait_for_rate_limit() {
        let client = Client::new(test_config()).await.unwrap();
        // Should return immediately when not rate-limited
        client.wait_for_rate_limit().await.unwrap();
    }
}
