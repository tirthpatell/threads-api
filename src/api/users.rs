use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::{
    PostsOptions, PostsResponse, PublicUser, RecentSearch, RepliesResponse, User, UserId,
};
use crate::validation;

impl Client {
    /// Get a user profile by ID.
    pub async fn get_user(&self, user_id: &UserId) -> crate::Result<User> {
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
        params.insert("fields".into(), constants::USER_PROFILE_FIELDS.into());

        let path = format!("/{}", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get the authenticated user's profile.
    pub async fn get_me(&self) -> crate::Result<User> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        self.get_user(&user_id).await
    }

    /// Look up a public user profile by username.
    pub async fn lookup_public_profile(&self, username: &str) -> crate::Result<PublicUser> {
        if username.is_empty() {
            return Err(error::new_validation_error(
                0,
                "Username is required",
                "",
                "username",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("username".into(), username.to_owned());
        params.insert("fields".into(), constants::PUBLIC_USER_FIELDS.into());

        let resp = self
            .http_client
            .get("/profile_lookup", params, &token)
            .await?;
        resp.json()
    }

    /// Get a user profile with custom field selection.
    pub async fn get_user_with_fields(
        &self,
        user_id: &UserId,
        fields: &[&str],
    ) -> crate::Result<User> {
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

        let fields_str = if fields.is_empty() {
            constants::USER_PROFILE_FIELDS.to_owned()
        } else {
            fields.join(",")
        };
        params.insert("fields".into(), fields_str);

        let path = format!("/{}", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get posts from a public user profile by username.
    pub async fn get_public_profile_posts(
        &self,
        username: &str,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<PostsResponse> {
        if username.is_empty() {
            return Err(error::new_validation_error(
                0,
                "Username is required",
                "",
                "username",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_posts_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("username".into(), username.to_owned());
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

        let resp = self
            .http_client
            .get("/profile_posts", params, &token)
            .await?;
        resp.json()
    }

    /// Get replies posted by a user.
    pub async fn get_user_replies(
        &self,
        user_id: &UserId,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<RepliesResponse> {
        if !user_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_USER_ID,
                "",
                "user_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_posts_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::REPLY_FIELDS.into());

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

        let path = format!("/{}/replies", user_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    // ---- Convenience methods using `me` ----

    /// Get the authenticated user's posts.
    pub async fn get_my_posts(&self, opts: Option<&PostsOptions>) -> crate::Result<PostsResponse> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        self.get_user_posts(&user_id, opts).await
    }

    /// Get the authenticated user's replies.
    pub async fn get_my_replies(
        &self,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<RepliesResponse> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        self.get_user_replies(&user_id, opts).await
    }

    /// Get posts where the authenticated user is mentioned.
    pub async fn get_my_mentions(
        &self,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<PostsResponse> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        self.get_user_mentions(&user_id, opts).await
    }

    /// Get the authenticated user's ghost posts.
    pub async fn get_my_ghost_posts(
        &self,
        opts: Option<&PostsOptions>,
    ) -> crate::Result<PostsResponse> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        self.get_user_ghost_posts(&user_id, opts).await
    }

    /// Get the authenticated user's recently searched keywords.
    pub async fn get_recently_searched_keywords(&self) -> crate::Result<Vec<RecentSearch>> {
        let uid = self.user_id().await;
        if uid.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "No user ID available from token",
            ));
        }
        let user_id = UserId::from(uid);
        let user = self
            .get_user_with_fields(&user_id, &["recently_searched_keywords"])
            .await?;
        Ok(user.recently_searched_keywords.unwrap_or_default())
    }
}
