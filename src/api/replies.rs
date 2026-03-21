use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::http::RequestBody;
use crate::types::{
    ImagePostContent, PendingRepliesOptions, Post, PostId, RepliesOptions, RepliesResponse,
    TextPostContent, VideoPostContent,
};
use crate::validation;

impl Client {
    /// Get replies to a post.
    pub async fn get_replies(
        &self,
        post_id: &PostId,
        opts: Option<&RepliesOptions>,
    ) -> crate::Result<RepliesResponse> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_replies_options(opts)?;
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
            if let Some(reverse) = opts.reverse {
                params.insert("reverse".into(), reverse.to_string());
            }
        }

        let path = format!("/{}/replies", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get the full conversation thread for a post.
    pub async fn get_conversation(
        &self,
        post_id: &PostId,
        opts: Option<&RepliesOptions>,
    ) -> crate::Result<RepliesResponse> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_replies_options(opts)?;
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
            if let Some(reverse) = opts.reverse {
                params.insert("reverse".into(), reverse.to_string());
            }
        }

        let path = format!("/{}/conversation", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Get pending replies awaiting moderation.
    pub async fn get_pending_replies(
        &self,
        post_id: &PostId,
        opts: Option<&PendingRepliesOptions>,
    ) -> crate::Result<RepliesResponse> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        if let Some(opts) = opts {
            validation::validate_pending_replies_options(opts)?;
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::PENDING_REPLY_FIELDS.into());

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
            if let Some(reverse) = opts.reverse {
                params.insert("reverse".into(), reverse.to_string());
            }
            if let Some(ref status) = opts.approval_status {
                params.insert(
                    "approval_status".into(),
                    serde_json::to_string(status)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned(),
                );
            }
        }

        let path = format!("/{}/pending_replies", post_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    /// Approve a pending reply.
    pub async fn approve_pending_reply(&self, reply_id: &PostId) -> crate::Result<()> {
        if !reply_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "reply_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("approve".into(), "true".into());

        let path = format!("/{}/manage_pending_reply", reply_id);
        let body = RequestBody::Form(params);
        self.http_client.post(&path, Some(body), &token).await?;
        Ok(())
    }

    /// Ignore a pending reply.
    pub async fn ignore_pending_reply(&self, reply_id: &PostId) -> crate::Result<()> {
        if !reply_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "reply_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("approve".into(), "false".into());

        let path = format!("/{}/manage_pending_reply", reply_id);
        let body = RequestBody::Form(params);
        self.http_client.post(&path, Some(body), &token).await?;
        Ok(())
    }

    /// Hide a reply.
    pub async fn hide_reply(&self, reply_id: &PostId) -> crate::Result<()> {
        if !reply_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "reply_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("hide".into(), "true".into());

        let path = format!("/{}/manage_reply", reply_id);
        let body = RequestBody::Form(params);
        self.http_client.post(&path, Some(body), &token).await?;
        Ok(())
    }

    /// Reply to a post with text.
    ///
    /// Convenience wrapper: builds a `TextPostContent` with `reply_to_id` set
    /// and delegates to `create_text_post`.
    pub async fn reply_to_post(&self, post_id: &PostId, text: &str) -> crate::Result<Post> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        let content = TextPostContent {
            text: text.to_owned(),
            reply_to_id: Some(post_id.clone()),
            link_attachment: None,
            poll_attachment: None,
            reply_control: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            auto_publish_text: false,
            quoted_post_id: None,
            text_entities: None,
            text_attachment: None,
            gif_attachment: None,
            is_ghost_post: false,
            enable_reply_approvals: false,
        };
        self.create_text_post(&content).await
    }

    /// Create a text reply.
    ///
    /// Validates that `reply_to_id` is set, then delegates to `create_text_post`.
    /// Set `apply_reply_delay` to `true` to apply the API-recommended 10-second
    /// delay before creating the reply.
    pub async fn create_reply(
        &self,
        content: &TextPostContent,
        apply_reply_delay: bool,
    ) -> crate::Result<Post> {
        if content.reply_to_id.is_none() {
            return Err(error::new_validation_error(
                0,
                "reply_to_id is required for create_reply",
                "",
                "reply_to_id",
            ));
        }

        if apply_reply_delay {
            tokio::time::sleep(constants::REPLY_PUBLISH_DELAY).await;
        }
        self.create_text_post(content).await
    }

    /// Create an image reply.
    ///
    /// Validates that `reply_to_id` is set, then delegates to `create_image_post`.
    pub async fn create_image_reply(&self, content: &ImagePostContent) -> crate::Result<Post> {
        if content.reply_to_id.is_none() {
            return Err(error::new_validation_error(
                0,
                "reply_to_id is required for create_image_reply",
                "",
                "reply_to_id",
            ));
        }

        self.create_image_post(content).await
    }

    /// Create a video reply.
    ///
    /// Validates that `reply_to_id` is set, then delegates to `create_video_post`.
    pub async fn create_video_reply(&self, content: &VideoPostContent) -> crate::Result<Post> {
        if content.reply_to_id.is_none() {
            return Err(error::new_validation_error(
                0,
                "reply_to_id is required for create_video_reply",
                "",
                "reply_to_id",
            ));
        }

        self.create_video_post(content).await
    }

    /// Unhide a reply.
    pub async fn unhide_reply(&self, reply_id: &PostId) -> crate::Result<()> {
        if !reply_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "reply_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("hide".into(), "false".into());

        let path = format!("/{}/manage_reply", reply_id);
        let body = RequestBody::Form(params);
        self.http_client.post(&path, Some(body), &token).await?;
        Ok(())
    }
}
