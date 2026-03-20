use serde::{Deserialize, Serialize};

/// A geographic location that can be tagged in posts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Location ID.
    pub id: String,
    /// Location name.
    pub name: String,
    /// Street address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// City name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Country name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Geographic latitude.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Geographic longitude.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Postal code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

/// Response from the location search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSearchResponse {
    /// List of matching locations.
    pub data: Vec<Location>,
}
