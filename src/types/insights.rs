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
    /// Post views count.
    #[serde(rename = "views")]
    Views,
    /// Post likes count.
    #[serde(rename = "likes")]
    Likes,
    /// Post replies count.
    #[serde(rename = "replies")]
    Replies,
    /// Post reposts count.
    #[serde(rename = "reposts")]
    Reposts,
    /// Post quotes count.
    #[serde(rename = "quotes")]
    Quotes,
    /// Post shares count.
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
    /// Account views count.
    #[serde(rename = "views")]
    Views,
    /// Account likes count.
    #[serde(rename = "likes")]
    Likes,
    /// Account replies count.
    #[serde(rename = "replies")]
    Replies,
    /// Account reposts count.
    #[serde(rename = "reposts")]
    Reposts,
    /// Account quotes count.
    #[serde(rename = "quotes")]
    Quotes,
    /// Account link clicks count.
    #[serde(rename = "clicks")]
    Clicks,
    /// Total followers count.
    #[serde(rename = "followers_count")]
    FollowersCount,
    /// Follower demographic breakdowns.
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
    /// Daily breakdown.
    #[serde(rename = "day")]
    Day,
    /// Lifetime aggregate.
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
    /// Breakdown by country.
    #[serde(rename = "country")]
    Country,
    /// Breakdown by city.
    #[serde(rename = "city")]
    City,
    /// Breakdown by age.
    #[serde(rename = "age")]
    Age,
    /// Breakdown by gender.
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
    /// Metrics to retrieve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<PostInsightMetric>>,
    /// Time period granularity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<InsightPeriod>,
    /// Start of the time range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    /// End of the time range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
}

/// Options for account insights requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountInsightsOptions {
    /// Metrics to retrieve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<AccountInsightMetric>>,
    /// Time period granularity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<InsightPeriod>,
    /// Start of the time range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    /// End of the time range.
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
    /// Metric name.
    pub name: String,
    /// Time period of the metric.
    pub period: String,
    /// Time-series values.
    #[serde(default)]
    pub values: Vec<Value>,
    /// Human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Insight ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Aggregated total value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_value: Option<TotalValue>,
}

/// A metric value with optional timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    /// The metric value.
    pub value: i64,
    /// End time of the measurement period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// An aggregated metric value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalValue {
    /// The aggregated value.
    pub value: i64,
    /// Link URL associated with this metric value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
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

    #[test]
    fn test_total_value_with_link_url() {
        let json = r#"{"value": 42, "link_url": "https://example.com"}"#;
        let tv: TotalValue = serde_json::from_str(json).unwrap();
        assert_eq!(tv.value, 42);
        assert_eq!(tv.link_url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_total_value_without_link_url() {
        let json = r#"{"value": 42}"#;
        let tv: TotalValue = serde_json::from_str(json).unwrap();
        assert_eq!(tv.value, 42);
        assert!(tv.link_url.is_none());
    }
}
