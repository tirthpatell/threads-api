use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::de::DeserializeOwned;

use crate::constants::{BASE_API_URL, VERSION};
use crate::error::{self, Error};
use crate::rate_limit::{RateLimitInfo, RateLimiter};

/// Body for HTTP requests.
#[derive(Debug)]
pub enum RequestBody {
    Json(serde_json::Value),
    Form(HashMap<String, String>),
}

/// Options for an HTTP request.
#[derive(Debug)]
pub struct RequestOptions {
    pub method: reqwest::Method,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub body: Option<RequestBody>,
    pub headers: HashMap<String, String>,
}

/// Response wrapper with metadata.
#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub body: Vec<u8>,
    pub request_id: String,
    pub rate_limit: Option<RateLimitInfo>,
    pub duration: Duration,
}

impl Response {
    /// Deserialize the response body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> crate::Result<T> {
        if self.body.is_empty() {
            return Err(error::new_api_error(
                self.status_code,
                "Empty response",
                "Received empty response body",
                &self.request_id,
            ));
        }

        let text = String::from_utf8_lossy(&self.body);
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(error::new_api_error(
                self.status_code,
                "Empty response",
                "Received whitespace-only response",
                &self.request_id,
            ));
        }

        if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            return Err(error::new_api_error(
                self.status_code,
                "Invalid JSON response",
                &format!(
                    "Received non-JSON response: {}",
                    &trimmed[..trimmed.len().min(200)]
                ),
                &self.request_id,
            ));
        }

        serde_json::from_slice(&self.body).map_err(Error::from)
    }
}

/// Retry configuration for HTTP requests.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

/// HTTP client with retry logic, rate limiting, and error handling.
pub struct HttpClient {
    client: reqwest::Client,
    retry_config: RetryConfig,
    rate_limiter: Option<Arc<RateLimiter>>,
    base_url: String,
    user_agent: String,
}

impl HttpClient {
    /// Create a new HTTP client.
    pub fn new(
        timeout: Duration,
        retry_config: RetryConfig,
        rate_limiter: Option<Arc<RateLimiter>>,
        base_url: Option<&str>,
        user_agent: Option<&str>,
    ) -> crate::Result<Self> {
        let ua = user_agent
            .map(|s| s.to_owned())
            .unwrap_or_else(|| format!("threads-api-rust/{}", VERSION));

        let client = reqwest::Client::builder().timeout(timeout).build()?;

        Ok(Self {
            client,
            retry_config,
            rate_limiter,
            base_url: base_url.unwrap_or(BASE_API_URL).to_owned(),
            user_agent: ua,
        })
    }

    /// Execute an HTTP request with retry logic and rate-limit awareness.
    pub async fn do_request(
        &self,
        opts: &RequestOptions,
        access_token: &str,
    ) -> crate::Result<Response> {
        let mut last_err: Option<Error> = None;
        let mut delay = self.retry_config.initial_delay;

        for attempt in 0..=self.retry_config.max_retries {
            // Check rate limiter before each attempt
            if let Some(ref rl) = self.rate_limiter {
                if rl.should_wait().await {
                    rl.wait().await?;
                }
            }

            if attempt > 0 {
                tokio::time::sleep(delay).await;
                delay = Duration::from_secs_f64(
                    delay.as_secs_f64() * self.retry_config.backoff_factor,
                );
                if delay > self.retry_config.max_delay {
                    delay = self.retry_config.max_delay;
                }
            }

            match self.execute_request(opts, access_token).await {
                Ok(resp) => {
                    if let (Some(rl), Some(info)) =
                        (&self.rate_limiter, &resp.rate_limit)
                    {
                        rl.update_from_headers(info).await;
                    }
                    return Ok(resp);
                }
                Err(err) => {
                    if !self.is_retryable_error(&err) {
                        return Err(err);
                    }
                    tracing::warn!(
                        attempt = attempt + 1,
                        max = self.retry_config.max_retries + 1,
                        error = %err,
                        "Retrying HTTP request"
                    );
                    last_err = Some(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            error::new_network_error(0, "Request failed after retries", "", false)
        }))
    }

    /// Execute a single HTTP request.
    async fn execute_request(
        &self,
        opts: &RequestOptions,
        access_token: &str,
    ) -> crate::Result<Response> {
        let start = Instant::now();

        let url = format!("{}{}", self.base_url, opts.path);

        let mut req = self.client.request(opts.method.clone(), &url);

        // Query parameters
        if !opts.query_params.is_empty() {
            req = req.query(
                &opts
                    .query_params
                    .iter()
                    .collect::<Vec<(&String, &String)>>(),
            );
        }

        // Standard headers
        req = req.header(USER_AGENT, &self.user_agent);
        if !access_token.is_empty() {
            req = req.header(AUTHORIZATION, format!("Bearer {}", access_token));
        }

        // Custom headers
        for (key, value) in &opts.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        // Body
        if let Some(ref body) = opts.body {
            match body {
                RequestBody::Json(val) => {
                    req = req.header(CONTENT_TYPE, "application/json");
                    req = req.json(val);
                }
                RequestBody::Form(params) => {
                    req = req.form(params);
                }
            }
        }

        tracing::debug!(method = %opts.method, path = %opts.path, "HTTP request");

        // Execute
        let http_resp = req.send().await.map_err(|e| self.wrap_network_error(e))?;
        let status = http_resp.status().as_u16();
        let request_id = http_resp
            .headers()
            .get("x-fb-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let rate_limit = Self::parse_rate_limit_headers(http_resp.headers());

        let body = http_resp
            .bytes()
            .await
            .map_err(|e| {
                error::new_network_error(
                    0,
                    "Failed to read response body",
                    &e.to_string(),
                    false,
                )
            })?
            .to_vec();

        let resp = Response {
            status_code: status,
            body,
            request_id,
            rate_limit,
            duration: start.elapsed(),
        };

        tracing::debug!(
            status = resp.status_code,
            duration_ms = resp.duration.as_millis() as u64,
            request_id = %resp.request_id,
            "HTTP response"
        );

        if status >= 400 {
            return Err(self.create_error_from_response(&resp).await);
        }

        Ok(resp)
    }

    /// Convenience: GET request.
    pub async fn get(
        &self,
        path: &str,
        query_params: HashMap<String, String>,
        access_token: &str,
    ) -> crate::Result<Response> {
        self.do_request(
            &RequestOptions {
                method: reqwest::Method::GET,
                path: path.to_owned(),
                query_params,
                body: None,
                headers: HashMap::new(),
            },
            access_token,
        )
        .await
    }

    /// Convenience: POST request with optional body.
    pub async fn post(
        &self,
        path: &str,
        body: Option<RequestBody>,
        access_token: &str,
    ) -> crate::Result<Response> {
        self.do_request(
            &RequestOptions {
                method: reqwest::Method::POST,
                path: path.to_owned(),
                query_params: HashMap::new(),
                body,
                headers: HashMap::new(),
            },
            access_token,
        )
        .await
    }

    /// Convenience: DELETE request.
    pub async fn delete(
        &self,
        path: &str,
        access_token: &str,
    ) -> crate::Result<Response> {
        self.do_request(
            &RequestOptions {
                method: reqwest::Method::DELETE,
                path: path.to_owned(),
                query_params: HashMap::new(),
                body: None,
                headers: HashMap::new(),
            },
            access_token,
        )
        .await
    }

    /// Parse rate limit info from response headers.
    fn parse_rate_limit_headers(headers: &HeaderMap) -> Option<RateLimitInfo> {
        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .unwrap_or(DateTime::UNIX_EPOCH);

        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);

        // No rate limit headers present
        if limit == 0 && remaining == 0 && reset == DateTime::UNIX_EPOCH {
            return None;
        }

        Some(RateLimitInfo {
            limit,
            remaining,
            reset,
            retry_after,
        })
    }

    /// Create a typed error from an API error response.
    async fn create_error_from_response(&self, resp: &Response) -> Error {
        #[derive(serde::Deserialize, Default)]
        struct ApiErrorResponse {
            #[serde(default)]
            error: ApiErrorBody,
        }

        #[derive(serde::Deserialize, Default)]
        struct ApiErrorBody {
            #[serde(default)]
            message: String,
            #[serde(default)]
            code: u16,
            #[serde(default)]
            is_transient: bool,
            #[serde(default)]
            error_subcode: u16,
        }

        let mut message = format!("HTTP {}", resp.status_code);
        let mut error_code = resp.status_code;
        let mut is_transient = false;
        let mut error_subcode: u16 = 0;

        if !resp.body.is_empty() {
            if let Ok(api_err) =
                serde_json::from_slice::<ApiErrorResponse>(&resp.body)
            {
                if !api_err.error.message.is_empty() {
                    message = api_err.error.message;
                    is_transient = api_err.error.is_transient;
                    error_subcode = api_err.error.error_subcode;
                    if api_err.error.code != 0 {
                        error_code = api_err.error.code;
                    }
                }
            }
        }

        let details = String::from_utf8_lossy(&resp.body);
        let details = if details.len() > 500 {
            let end = details
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= 500)
                .last()
                .unwrap_or(0);
            format!("{}...", &details[..end])
        } else {
            details.into_owned()
        };

        let mut err = match resp.status_code {
            401 | 403 => {
                error::new_authentication_error(error_code, &message, &details)
            }
            429 => {
                let retry_after = resp
                    .rate_limit
                    .as_ref()
                    .and_then(|rl| rl.retry_after)
                    .unwrap_or(Duration::ZERO);

                if let Some(ref rl) = self.rate_limiter {
                    let reset_time = resp
                        .rate_limit
                        .as_ref()
                        .map(|rl| rl.reset)
                        .filter(|t| *t > Utc::now())
                        .unwrap_or_else(|| {
                            Utc::now()
                                + chrono::Duration::from_std(retry_after)
                                    .unwrap_or(chrono::Duration::seconds(60))
                        });
                    rl.mark_rate_limited(reset_time).await;
                }

                error::new_rate_limit_error(
                    error_code,
                    &message,
                    &details,
                    retry_after,
                )
            }
            400 | 422 => {
                error::new_validation_error(error_code, &message, &details, "")
            }
            _ => error::new_api_error(
                error_code,
                &message,
                &details,
                &resp.request_id,
            ),
        };

        error::set_error_metadata(
            &mut err,
            is_transient,
            resp.status_code,
            error_subcode,
        );

        err
    }

    /// Check if an error should trigger a retry.
    fn is_retryable_error(&self, err: &Error) -> bool {
        if err.is_retryable() {
            return true;
        }
        // Also retry 5xx server errors
        if let Some(fields) = error::extract_base_fields(err) {
            if fields.http_status_code >= 500 && fields.http_status_code < 600 {
                return true;
            }
        }
        false
    }

    /// Wrap a reqwest error into our typed errors.
    fn wrap_network_error(&self, err: reqwest::Error) -> Error {
        if err.is_timeout() {
            return error::new_network_error_with_cause(
                0,
                "Request timeout",
                &err.to_string(),
                true,
                Some(err),
            );
        }
        if err.is_connect() {
            return error::new_network_error_with_cause(
                0,
                "Connection error",
                &err.to_string(),
                true,
                Some(err),
            );
        }
        error::new_network_error_with_cause(
            0,
            "Network error",
            &err.to_string(),
            false,
            Some(err),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.initial_delay, Duration::from_secs(1));
        assert_eq!(cfg.max_delay, Duration::from_secs(30));
        assert!((cfg.backoff_factor - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_response_json_empty_body() {
        let resp = Response {
            status_code: 200,
            body: vec![],
            request_id: "test".to_owned(),
            rate_limit: None,
            duration: Duration::ZERO,
        };
        let result: Result<serde_json::Value, _> = resp.json();
        assert!(result.is_err());
    }

    #[test]
    fn test_response_json_valid() {
        let resp = Response {
            status_code: 200,
            body: br#"{"id":"123"}"#.to_vec(),
            request_id: "test".to_owned(),
            rate_limit: None,
            duration: Duration::ZERO,
        };
        let val: serde_json::Value = resp.json().unwrap();
        assert_eq!(val["id"], "123");
    }

    #[test]
    fn test_response_json_non_json() {
        let resp = Response {
            status_code: 200,
            body: b"not json at all".to_vec(),
            request_id: "test".to_owned(),
            rate_limit: None,
            duration: Duration::ZERO,
        };
        let result: Result<serde_json::Value, _> = resp.json();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rate_limit_headers_empty() {
        let headers = HeaderMap::new();
        assert!(HttpClient::parse_rate_limit_headers(&headers).is_none());
    }

    #[test]
    fn test_parse_rate_limit_headers_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-limit", "100".parse().unwrap());
        headers.insert("x-ratelimit-remaining", "42".parse().unwrap());
        headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());
        headers.insert("retry-after", "60".parse().unwrap());

        let info = HttpClient::parse_rate_limit_headers(&headers).unwrap();
        assert_eq!(info.limit, 100);
        assert_eq!(info.remaining, 42);
        assert_eq!(info.retry_after, Some(Duration::from_secs(60)));
    }

    #[tokio::test]
    async fn test_http_client_new() {
        let client = HttpClient::new(
            Duration::from_secs(30),
            RetryConfig::default(),
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(client.base_url, BASE_API_URL);
        assert!(client.user_agent.starts_with("threads-api-rust/"));
    }
}
