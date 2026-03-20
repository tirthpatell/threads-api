use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Earliest Unix timestamp usable for insights queries.
pub const MIN_INSIGHT_TIMESTAMP: i64 = 1_712_991_600;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Available post insight metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostInsightMetric {
    #[serde(rename = "views")]
    Views,
    #[serde(rename = "likes")]
    Likes,
    #[serde(rename = "replies")]
    Replies,
    #[serde(rename = "reposts")]
    Reposts,
    #[serde(rename = "quotes")]
    Quotes,
    #[serde(rename = "shares")]
    Shares,
}

impl PostInsightMetric {
    /// Returns a slice of all variants.
    pub fn all() -> &'static [Self] {
        &[
            Self::Views,
            Self::Likes,
            Self::Replies,
            Self::Reposts,
            Self::Quotes,
            Self::Shares,
        ]
    }
}

/// Available account insight metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountInsightMetric {
    #[serde(rename = "views")]
    Views,
    #[serde(rename = "likes")]
    Likes,
    #[serde(rename = "replies")]
    Replies,
    #[serde(rename = "reposts")]
    Reposts,
    #[serde(rename = "quotes")]
    Quotes,
    #[serde(rename = "clicks")]
    Clicks,
    #[serde(rename = "followers_count")]
    FollowersCount,
    #[serde(rename = "follower_demographics")]
    FollowerDemographics,
}

/// Available account insight metrics.
impl AccountInsightMetric {
    /// Returns a slice of all variants.
    pub fn all() -> &'static [Self] {
        &[
            Self::Views,
            Self::Likes,
            Self::Replies,
            Self::Reposts,
            Self::Quotes,
            Self::Clicks,
            Self::FollowersCount,
            Self::FollowerDemographics,
        ]
    }
}

/// Time period for insights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightPeriod {
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "lifetime")]
    Lifetime,
}

impl InsightPeriod {
    /// Returns a slice of all variants.
    pub fn all() -> &'static [Self] {
        &[Self::Day, Self::Lifetime]
    }
}

/// Breakdown options for follower demographics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowerDemographicsBreakdown {
    #[serde(rename = "country")]
    Country,
    #[serde(rename = "city")]
    City,
    #[serde(rename = "age")]
    Age,
    #[serde(rename = "gender")]
    Gender,
}

impl FollowerDemographicsBreakdown {
    /// Returns a slice of all variants.
    pub fn all() -> &'static [Self] {
        &[Self::Country, Self::City, Self::Age, Self::Gender]
    }
}

// ---------------------------------------------------------------------------
// Option structs
// ---------------------------------------------------------------------------

/// Options for post insights requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostInsightsOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<PostInsightMetric>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<InsightPeriod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
}

/// Options for account insights requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountInsightsOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<AccountInsightMetric>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<InsightPeriod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
    /// For `follower_demographics`: country, city, age, or gender.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<FollowerDemographicsBreakdown>,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

/// An individual analytics metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub name: String,
    pub period: String,
    #[serde(default)]
    pub values: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_value: Option<TotalValue>,
}

/// A metric value with optional timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub value: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// An aggregated metric value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalValue {
    pub value: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_insight_metric_all() {
        let all = PostInsightMetric::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], PostInsightMetric::Views);
        assert_eq!(all[5], PostInsightMetric::Shares);
    }

    #[test]
    fn test_account_insight_metric_all() {
        let all = AccountInsightMetric::all();
        assert_eq!(all.len(), 8);
        assert_eq!(all[0], AccountInsightMetric::Views);
        assert_eq!(all[7], AccountInsightMetric::FollowerDemographics);
    }

    #[test]
    fn test_insight_period_all() {
        let all = InsightPeriod::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], InsightPeriod::Day);
        assert_eq!(all[1], InsightPeriod::Lifetime);
    }

    #[test]
    fn test_follower_demographics_breakdown_all() {
        let all = FollowerDemographicsBreakdown::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], FollowerDemographicsBreakdown::Country);
        assert_eq!(all[3], FollowerDemographicsBreakdown::Gender);
    }

    #[test]
    fn test_post_insight_metric_serde() {
        let metric = PostInsightMetric::Views;
        let json = serde_json::to_string(&metric).unwrap();
        assert_eq!(json, r#""views""#);
        let back: PostInsightMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PostInsightMetric::Views);
    }

    #[test]
    fn test_insight_period_serde() {
        let period = InsightPeriod::Day;
        let json = serde_json::to_string(&period).unwrap();
        assert_eq!(json, r#""day""#);
    }

    #[test]
    fn test_post_insights_options_default() {
        let opts = PostInsightsOptions::default();
        assert!(opts.metrics.is_none());
        assert!(opts.period.is_none());
        assert!(opts.since.is_none());
        assert!(opts.until.is_none());
    }

    #[test]
    fn test_account_insights_options_default() {
        let opts = AccountInsightsOptions::default();
        assert!(opts.metrics.is_none());
        assert!(opts.breakdown.is_none());
    }
}
