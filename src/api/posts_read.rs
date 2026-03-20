use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::{
    PaginationOptions, Post, PostId, PostsOptions, PostsResponse, PublishingLimits, UserId,
};
use crate::validation;

impl Client {
    /// Get a single post by ID with extended fields.
    pub async fn get_post(&self, post_id: &PostId) -> crate::Result<Post> {
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
        params.insert("fields".into(), constants::POST_EXTENDED_FIELDS.into());

        let path = format!("/{}", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get posts created by a user.
    pub async fn get_user_posts(
        &self,
        user_id: &UserId,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<PostsResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_limit(opts.limit)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::POST_EXTENDED_FIELDS.into());

        if let Some(opts) = opts {
            if let Some(limit) = opts.limit {
                params.insert("limit".into(), limit.to_string());
            }
            if let Some(ref before) = opts.before {
                params.insert("before".into(), before.clone());
            }
            if let Some(ref after) = opts.after {
                params.insert("after".into(), after.clone());
            }
            if let Some(since) = opts.since {
                params.insert("since".into(), since.to_string());
            }
            if let Some(until) = opts.until {
                params.insert("until".into(), until.to_string());
            }
        }

        let path = format!("/{}/threads", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get posts where the user is mentioned.
    pub async fn get_user_mentions(
        &self,
        user_id: &UserId,
        opts: Option<&PaginationOptions>,
    ) -> crate::Result<PostsResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_pagination_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::POST_EXTENDED_FIELDS.into());

        if let Some(opts) = opts {
            if let Some(limit) = opts.limit {
                params.insert("limit".into(), limit.to_string());
            }
            if let Some(ref before) = opts.before {
                params.insert("before".into(), before.clone());
            }
            if let Some(ref after) = opts.after {
                params.insert("after".into(), after.clone());
            }
        }

        let path = format!("/{}/mentions", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get ghost posts for a user.
    pub async fn get_user_ghost_posts(
        &self,
        user_id: &UserId,
        opts: Option<&PaginationOptions>,
    ) -> crate::Result<PostsResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_pagination_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::GHOST_POST_FIELDS.into());

        if let Some(opts) = opts {
            if let Some(limit) = opts.limit {
                params.insert("limit".into(), limit.to_string());
            }
            if let Some(ref before) = opts.before {
                params.insert("before".into(), before.clone());
            }
            if let Some(ref after) = opts.after {
                params.insert("after".into(), after.clone());
            }
        }

        let path = format!("/{}/ghost_posts", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get the publishing rate limits for the authenticated user.
    pub async fn get_publishing_limits(&self) -> crate::Result<PublishingLimits> {
        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::PUBLISHING_LIMIT_FIELDS.into());

        let path = format!("/{}/threads_publishing_limit", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;

        // API wraps publishing limits in {"data": [PublishingLimits]}
        #[derive(serde::Deserialize)]
        struct Wrapper {
            data: Vec<PublishingLimits>,
        }

        let wrapper: Wrapper = resp.json()?;
        wrapper
            .data
            .into_iter()
            .next()
            .ok_or_else(|| error::new_api_error(0, "No publishing limits data returned", "", ""))
    }
}
