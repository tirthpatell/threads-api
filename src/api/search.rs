use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::PostsResponse;
use crate::types::search::SearchOptions;
use crate::validation;

impl Client {
    /// Search for posts by keyword.
    pub async fn keyword_search(
        &self,
        query: &str,
        opts: Option<&SearchOptions>,
    ) -> crate::Result<PostsResponse> {
        if query.is_empty() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_SEARCH_QUERY,
                "",
                "query",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_search_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("q".into(), query.to_owned());
        params.insert("fields".into(), constants::POST_EXTENDED_FIELDS.into());

        if let Some(opts) = opts {
            if let Some(ref search_type) = opts.search_type {
                params.insert(
                    "search_type".into(),
                    serde_json::to_string(search_type)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned(),
                );
            }
            if let Some(ref search_mode) = opts.search_mode {
                params.insert(
                    "search_mode".into(),
                    serde_json::to_string(search_mode)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned(),
                );
            }
            if let Some(ref media_type) = opts.media_type {
                params.insert("media_type".into(), media_type.clone());
            }
            if let Some(ref author) = opts.author_username {
                params.insert("author_username".into(), author.clone());
            }
            if let Some(limit) = opts.limit {
                params.insert("limit".into(), limit.to_string());
            }
            if let Some(since) = opts.since {
                params.insert("since".into(), since.to_string());
            }
            if let Some(until) = opts.until {
                params.insert("until".into(), until.to_string());
            }
            if let Some(ref before) = opts.before {
                params.insert("before".into(), before.clone());
            }
            if let Some(ref after) = opts.after {
                params.insert("after".into(), after.clone());
            }
        }

        let resp = self
            .http_client
            .get("/keyword_search", params, &token)
            .await?;
        resp.json()
    }
}
