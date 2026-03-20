use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::http::RequestBody;
use crate::types::{PendingRepliesOptions, PostId, RepliesOptions, RepliesResponse};

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
