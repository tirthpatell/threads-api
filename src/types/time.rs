use chrono::{DateTime, FixedOffset, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Custom time type that handles various Threads API timestamp formats.
///
/// The API may return timestamps in several formats. This type tries them all
/// during deserialization and always serialises to RFC 3339.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadsTime(pub DateTime<Utc>);

impl ThreadsTime {
    /// Create a new `ThreadsTime` from a `DateTime<Utc>`.
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl fmt::Display for ThreadsTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl Serialize for ThreadsTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for ThreadsTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        // Threads API format: "2006-01-02T15:04:05+0000"
        if let Ok(dt) = DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%z") {
            return Ok(ThreadsTime(dt.with_timezone(&Utc)));
        }

        // ISO 8601 UTC: "2006-01-02T15:04:05Z"
        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
            return Ok(ThreadsTime(dt));
        }

        // RFC 3339
        if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
            return Ok(ThreadsTime(dt.with_timezone(&Utc)));
        }

        // With timezone offset: "2006-01-02T15:04:05-0700"
        if let Ok(dt) = DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%z") {
            return Ok(ThreadsTime(dt.with_timezone(&Utc)));
        }

        // Fallback: try chrono's flexible parsing
        if let Ok(dt) = DateTime::<FixedOffset>::parse_from_rfc3339(&s) {
            return Ok(ThreadsTime(dt.with_timezone(&Utc)));
        }

        Err(serde::de::Error::custom(format!(
            "unable to parse timestamp: {s}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threads_api_format() {
        let json = r#""2024-01-15T10:30:00+0000""#;
        let t: ThreadsTime = serde_json::from_str(json).unwrap();
        assert_eq!(t.0.year(), 2024);
        assert_eq!(t.0.month(), 1);
        assert_eq!(t.0.day(), 15);
    }

    #[test]
    fn test_iso8601_utc() {
        let json = r#""2024-06-01T12:00:00Z""#;
        let t: ThreadsTime = serde_json::from_str(json).unwrap();
        assert_eq!(t.0.year(), 2024);
        assert_eq!(t.0.month(), 6);
    }

    #[test]
    fn test_rfc3339() {
        let json = r#""2024-03-20T08:15:30+05:30""#;
        let t: ThreadsTime = serde_json::from_str(json).unwrap();
        assert_eq!(t.0.year(), 2024);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let json = r#""2024-01-15T10:30:00+0000""#;
        let t: ThreadsTime = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&t).unwrap();
        // Should serialize to RFC 3339
        let back: ThreadsTime = serde_json::from_str(&serialized).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn test_display() {
        let json = r#""2024-01-01T00:00:00Z""#;
        let t: ThreadsTime = serde_json::from_str(json).unwrap();
        let s = t.to_string();
        assert!(s.contains("2024-01-01"));
    }

    #[test]
    fn test_invalid_timestamp() {
        let json = r#""not-a-timestamp""#;
        let result = serde_json::from_str::<ThreadsTime>(json);
        assert!(result.is_err());
    }

    use chrono::Datelike;
}
