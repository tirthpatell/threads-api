use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Rate limit information extracted from API response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: DateTime<Utc>,
    pub retry_after: Option<Duration>,
}

/// Snapshot of the current rate limit status.
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub reset_in: Duration,
}

/// Configuration for the rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub initial_limit: u32,
    pub backoff_multiplier: f64,
    pub max_backoff: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            initial_limit: 100,
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(300),
        }
    }
}

struct Inner {
    limit: u32,
    remaining: u32,
    reset_time: DateTime<Utc>,
    last_request_time: Option<Instant>,
    backoff_multiplier: f64,
    max_backoff: Duration,
    rate_limited: bool,
    last_rate_limit_time: Option<Instant>,
    consecutive_rate_limits: u32,
}

/// Manages API rate limiting with intelligent backoff.
///
/// Thread-safe via internal `RwLock`. All methods take `&self`.
pub struct RateLimiter {
    inner: RwLock<Inner>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: &RateLimiterConfig) -> Self {
        let limit = if config.initial_limit == 0 {
            100
        } else {
            config.initial_limit
        };
        let backoff = if config.backoff_multiplier <= 0.0 {
            2.0
        } else {
            config.backoff_multiplier
        };
        let max_backoff = if config.max_backoff.is_zero() {
            Duration::from_secs(300)
        } else {
            config.max_backoff
        };

        Self {
            inner: RwLock::new(Inner {
                limit,
                remaining: limit,
                reset_time: Utc::now() + chrono::Duration::hours(1),
                last_request_time: None,
                backoff_multiplier: backoff,
                max_backoff,
                rate_limited: false,
                last_rate_limit_time: None,
                consecutive_rate_limits: 0,
            }),
        }
    }

    /// Returns `true` if a request should wait before proceeding.
    /// Only returns `true` when the API has explicitly rate-limited us.
    pub async fn should_wait(&self) -> bool {
        let inner = self.inner.read().await;
        inner.rate_limited && Utc::now() < inner.reset_time
    }

    /// Blocks until it's safe to make a request.
    /// Only blocks when actually rate-limited by the API.
    pub async fn wait(&self) -> crate::Result<()> {
        // Check if window has reset
        {
            let mut inner = self.inner.write().await;
            if Utc::now() >= inner.reset_time {
                inner.remaining = inner.limit;
                inner.reset_time = Utc::now() + chrono::Duration::hours(1);
                inner.rate_limited = false;
                inner.consecutive_rate_limits = 0;
                tracing::debug!(limit = inner.limit, "Rate limit window reset");
                return Ok(());
            }
            if !inner.rate_limited {
                inner.last_request_time = Some(Instant::now());
                return Ok(());
            }
        }

        // Rate-limited: sleep until reset
        loop {
            let (wait_duration, original_reset) = {
                let inner = self.inner.read().await;
                let mut wait_time = (inner.reset_time - Utc::now())
                    .to_std()
                    .unwrap_or(Duration::from_secs(1));

                // Apply exponential backoff if hitting limits repeatedly
                if inner.consecutive_rate_limits > 1 {
                    let base_delay = Duration::from_secs(1);
                    let exponent = (inner.consecutive_rate_limits - 1).min(10);
                    let backoff_secs = base_delay.as_secs_f64()
                        * inner.backoff_multiplier.powi(exponent as i32);
                    let backoff = Duration::from_secs_f64(backoff_secs);
                    if backoff > wait_time {
                        wait_time = backoff;
                    }
                    if wait_time > inner.max_backoff {
                        wait_time = inner.max_backoff;
                    }
                }

                tracing::info!(
                    wait_ms = wait_time.as_millis() as u64,
                    remaining = inner.remaining,
                    "API rate limit enforced, waiting"
                );

                (wait_time, inner.reset_time)
            };

            tokio::time::sleep(wait_duration).await;

            // Check if rate limit was extended while sleeping
            let mut inner = self.inner.write().await;
            if inner.reset_time > original_reset {
                continue;
            }
            inner.rate_limited = false;
            inner.last_request_time = Some(Instant::now());
            return Ok(());
        }
    }

    /// Updates rate limit state from API response headers.
    pub async fn update_from_headers(&self, info: &RateLimitInfo) {
        let mut inner = self.inner.write().await;
        if info.limit > 0 {
            inner.limit = info.limit;
        }
        inner.remaining = info.remaining;
        if info.reset > Utc::now() {
            inner.reset_time = info.reset;
        }
        tracing::debug!(
            limit = info.limit,
            remaining = info.remaining,
            "Rate limit updated from headers"
        );
    }

    /// Marks that the API has returned a 429 rate-limit response.
    pub async fn mark_rate_limited(&self, reset_time: DateTime<Utc>) {
        let mut inner = self.inner.write().await;
        inner.rate_limited = true;
        inner.last_rate_limit_time = Some(Instant::now());
        inner.consecutive_rate_limits += 1;
        if reset_time > Utc::now() {
            inner.reset_time = reset_time;
        }
        tracing::info!(
            consecutive = inner.consecutive_rate_limits,
            "Marked as rate limited by API"
        );
    }

    /// Returns a snapshot of the current rate limit status.
    pub async fn get_status(&self) -> RateLimitStatus {
        let inner = self.inner.read().await;
        let reset_in = (inner.reset_time - Utc::now())
            .to_std()
            .unwrap_or(Duration::ZERO);
        RateLimitStatus {
            limit: inner.limit,
            remaining: inner.remaining,
            reset_time: inner.reset_time,
            reset_in,
        }
    }

    /// Returns `true` if usage has exceeded the given threshold (0.0–1.0).
    pub async fn is_near_limit(&self, threshold: f64) -> bool {
        let inner = self.inner.read().await;
        if inner.limit == 0 {
            return false;
        }
        let used = inner.limit.saturating_sub(inner.remaining) as f64 / inner.limit as f64;
        used >= threshold
    }

    /// Returns `true` if the API has rate-limited us and the window hasn't reset.
    pub async fn is_rate_limited(&self) -> bool {
        let inner = self.inner.read().await;
        inner.rate_limited && Utc::now() < inner.reset_time
    }

    /// Resets the rate limiter to its initial state.
    pub async fn reset(&self) {
        let mut inner = self.inner.write().await;
        inner.remaining = inner.limit;
        inner.reset_time = Utc::now() + chrono::Duration::hours(1);
        inner.last_request_time = None;
        inner.rate_limited = false;
        inner.consecutive_rate_limits = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_default_config() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        let status = rl.get_status().await;
        assert_eq!(status.limit, 100);
        assert_eq!(status.remaining, 100);
    }

    #[tokio::test]
    async fn test_should_wait_initially_false() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        assert!(!rl.should_wait().await);
    }

    #[tokio::test]
    async fn test_mark_rate_limited() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        assert!(!rl.is_rate_limited().await);
        let reset = Utc::now() + chrono::Duration::minutes(5);
        rl.mark_rate_limited(reset).await;
        assert!(rl.is_rate_limited().await);
        assert!(rl.should_wait().await);
    }

    #[tokio::test]
    async fn test_update_from_headers() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        let info = RateLimitInfo {
            limit: 200,
            remaining: 150,
            reset: Utc::now() + chrono::Duration::hours(1),
            retry_after: None,
        };
        rl.update_from_headers(&info).await;
        let status = rl.get_status().await;
        assert_eq!(status.limit, 200);
        assert_eq!(status.remaining, 150);
    }

    #[tokio::test]
    async fn test_is_near_limit() {
        let rl = RateLimiter::new(&RateLimiterConfig {
            initial_limit: 100,
            ..Default::default()
        });
        assert!(!rl.is_near_limit(0.8).await);
        let info = RateLimitInfo {
            limit: 100,
            remaining: 10,
            reset: Utc::now() + chrono::Duration::hours(1),
            retry_after: None,
        };
        rl.update_from_headers(&info).await;
        assert!(rl.is_near_limit(0.8).await);
    }

    #[tokio::test]
    async fn test_reset() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        rl.mark_rate_limited(Utc::now() + chrono::Duration::minutes(5))
            .await;
        assert!(rl.is_rate_limited().await);
        rl.reset().await;
        assert!(!rl.is_rate_limited().await);
        let status = rl.get_status().await;
        assert_eq!(status.remaining, status.limit);
    }

    #[tokio::test]
    async fn test_wait_not_rate_limited() {
        let rl = RateLimiter::new(&RateLimiterConfig::default());
        // Should return immediately when not rate-limited
        rl.wait().await.unwrap();
    }
}
