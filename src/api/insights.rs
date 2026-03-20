use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::{AccountInsightsOptions, InsightsResponse, PostId, PostInsightsOptions, UserId};

impl Client {
    /// Get insights for a specific post.
    pub async fn get_post_insights(
        &self,
        post_id: &PostId,
        metrics: &[&str],
    ) -> crate::Result<InsightsResponse> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();

        let metric_str = if metrics.is_empty() {
            "views,likes,replies,reposts,quotes".to_owned()
        } else {
            metrics.join(",")
        };
        params.insert("metric".into(), metric_str);

        let path = format!("/{}/insights", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get insights for a user account.
    pub async fn get_account_insights(
        &self,
        user_id: &UserId,
        metrics: &[&str],
        period: &str,
    ) -> crate::Result<InsightsResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();

        let metric_str = if metrics.is_empty() {
            "views,reach,replies,reposts,quotes,followers_count".to_owned()
        } else {
            metrics.join(",")
        };
        params.insert("metric".into(), metric_str);

        let period_str = if period.is_empty() {
            "lifetime".to_owned()
        } else {
            period.to_owned()
        };
        params.insert("period".into(), period_str);

        let path = format!("/{}/threads_insights", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get post insights with typed options.
    pub async fn get_post_insights_with_options(
        &self,
        post_id: &PostId,
        opts: &PostInsightsOptions,
    ) -> crate::Result<InsightsResponse> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();

        let metric_str = match &opts.metrics {
            Some(metrics) if !metrics.is_empty() => metrics
                .iter()
                .map(|m| {
                    serde_json::to_string(m)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned()
                })
                .collect::<Vec<_>>()
                .join(","),
            _ => "views,likes,replies,reposts,quotes".to_owned(),
        };
        params.insert("metric".into(), metric_str);

        if let Some(ref period) = opts.period {
            params.insert(
                "period".into(),
                serde_json::to_string(period)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }

        if let Some(since) = opts.since {
            params.insert("since".into(), since.timestamp().to_string());
        }
        if let Some(until) = opts.until {
            params.insert("until".into(), until.timestamp().to_string());
        }

        let path = format!("/{}/insights", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get account insights with typed options.
    pub async fn get_account_insights_with_options(
        &self,
        user_id: &UserId,
        opts: &AccountInsightsOptions,
    ) -> crate::Result<InsightsResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();

        let metric_str = match &opts.metrics {
            Some(metrics) if !metrics.is_empty() => metrics
                .iter()
                .map(|m| {
                    serde_json::to_string(m)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned()
                })
                .collect::<Vec<_>>()
                .join(","),
            _ => "views,reach,replies,reposts,quotes,followers_count".to_owned(),
        };
        params.insert("metric".into(), metric_str);

        let period_str = match &opts.period {
            Some(period) => serde_json::to_string(period)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned(),
            None => "lifetime".to_owned(),
        };
        params.insert("period".into(), period_str);

        if let Some(since) = opts.since {
            params.insert("since".into(), since.timestamp().to_string());
        }
        if let Some(until) = opts.until {
            params.insert("until".into(), until.timestamp().to_string());
        }
        if let Some(ref breakdown) = opts.breakdown {
            params.insert(
                "breakdown".into(),
                serde_json::to_string(breakdown)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }

        let path = format!("/{}/threads_insights", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }
}
