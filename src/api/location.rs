use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::{Location, LocationId, LocationSearchResponse};

impl Client {
    /// Search for locations by query string, coordinates, or both.
    ///
    /// Per the API docs, you can search by `query` alone, `latitude` + `longitude`
    /// alone, or all three together. At least one of `query` or coordinates must
    /// be provided.
    pub async fn search_locations(
        &self,
        query: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> crate::Result<LocationSearchResponse> {
        let has_query = query.is_some_and(|q| !q.is_empty());
        let has_coords = latitude.is_some() && longitude.is_some();

        if !has_query && !has_coords {
            return Err(error::new_validation_error(
                0,
                "Either a search query or latitude+longitude coordinates are required",
                "",
                "query",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::LOCATION_FIELDS.into());

        if let Some(q) = query {
            if !q.is_empty() {
                params.insert("q".into(), q.to_owned());
            }
        }

        if let Some(lat) = latitude {
            params.insert("latitude".into(), lat.to_string());
        }
        if let Some(lng) = longitude {
            params.insert("longitude".into(), lng.to_string());
        }

        let resp = self
            .http_client
            .get("/location_search", params, &token)
            .await?;
        resp.json()
    }

    /// Get a location by ID.
    pub async fn get_location(&self, location_id: &LocationId) -> crate::Result<Location> {
        if !location_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                "Location ID is required",
                "",
                "location_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::LOCATION_FIELDS.into());

        let path = format!("/{}", location_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }
}
