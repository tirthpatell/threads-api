use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::{InsightsResponse, PostId, UserId};

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
}
