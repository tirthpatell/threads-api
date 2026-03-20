use std::collections::HashMap;

use crate::client::Client;
use crate::constants;
use crate::error;
use crate::http::RequestBody;
use crate::types::{
    CarouselPostContent, ContainerId, ContainerStatus, ImagePostContent, Post, PostId,
    TextPostContent, VideoPostContent,
};
use crate::validation;

impl Client {
    /// Create a text post. If `auto_publish_text` is true, skips the
    /// container+publish flow and posts directly.
    pub async fn create_text_post(&self, content: &TextPostContent) -> crate::Result<Post> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "",
            ));
        }

        if content.text.is_empty() {
            return Err(error::new_validation_error(
                0,
                "Text is required for text posts",
                "",
                "text",
            ));
        }

        validation::validate_text_length(&content.text, "text")?;
        validation::validate_link_count(
            &content.text,
            content.link_attachment.as_deref().unwrap_or(""),
        )?;
        if let Some(ref entities) = content.text_entities {
            validation::validate_text_entities(entities, content.text.chars().count())?;
        }
        if let Some(ref attachment) = content.text_attachment {
            validation::validate_text_attachment(attachment)?;
        }
        if let Some(ref gif) = content.gif_attachment {
            validation::validate_gif_attachment(gif)?;
        }
        if let Some(ref tag) = content.topic_tag {
            validation::validate_topic_tag(tag)?;
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            validation::validate_country_codes(codes)?;
        }

        if content.auto_publish_text {
            // Text containers are ready immediately — skip polling
            let params = self.build_text_params(content, &user_id);
            let container_id = self.create_container(params).await?;
            return self.publish_container(&container_id).await;
        }

        let params = self.build_text_params(content, &user_id);
        let container_id = self.create_container(params).await?;
        let cid = ContainerId::from(container_id.as_str());
        self.wait_for_container_ready(&cid).await?;
        self.publish_container(&container_id).await
    }

    /// Create an image post.
    pub async fn create_image_post(&self, content: &ImagePostContent) -> crate::Result<Post> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "",
            ));
        }

        validation::validate_media_url(&content.image_url, "image")?;
        if let Some(ref text) = content.text {
            validation::validate_text_length(text, "text")?;
        }
        if let Some(ref entities) = content.text_entities {
            let text_len = content.text.as_deref().map_or(0, |t| t.chars().count());
            validation::validate_text_entities(entities, text_len)?;
        }
        if let Some(ref tag) = content.topic_tag {
            validation::validate_topic_tag(tag)?;
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            validation::validate_country_codes(codes)?;
        }

        let params = self.build_image_params(content, &user_id);
        let container_id = self.create_container(params).await?;
        let cid = ContainerId::from(container_id.as_str());
        self.wait_for_container_ready(&cid).await?;
        self.publish_container(&container_id).await
    }

    /// Create a video post.
    pub async fn create_video_post(&self, content: &VideoPostContent) -> crate::Result<Post> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "",
            ));
        }

        validation::validate_media_url(&content.video_url, "video")?;
        if let Some(ref text) = content.text {
            validation::validate_text_length(text, "text")?;
        }
        if let Some(ref entities) = content.text_entities {
            let text_len = content.text.as_deref().map_or(0, |t| t.chars().count());
            validation::validate_text_entities(entities, text_len)?;
        }
        if let Some(ref tag) = content.topic_tag {
            validation::validate_topic_tag(tag)?;
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            validation::validate_country_codes(codes)?;
        }

        let params = self.build_video_params(content, &user_id);
        let container_id = self.create_container(params).await?;
        let cid = ContainerId::from(container_id.as_str());
        self.wait_for_container_ready(&cid).await?;
        self.publish_container(&container_id).await
    }

    /// Create a carousel post.
    pub async fn create_carousel_post(&self, content: &CarouselPostContent) -> crate::Result<Post> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "",
            ));
        }

        validation::validate_carousel_children(content.children.len())?;
        if let Some(ref text) = content.text {
            validation::validate_text_length(text, "text")?;
        }
        if let Some(ref entities) = content.text_entities {
            let text_len = content.text.as_deref().map_or(0, |t| t.chars().count());
            validation::validate_text_entities(entities, text_len)?;
        }
        if let Some(ref tag) = content.topic_tag {
            validation::validate_topic_tag(tag)?;
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            validation::validate_country_codes(codes)?;
        }

        let params = self.build_carousel_params(content, &user_id);
        let container_id = self.create_container(params).await?;
        let cid = ContainerId::from(container_id.as_str());
        self.wait_for_container_ready(&cid).await?;
        self.publish_container(&container_id).await
    }

    /// Create a quote post — a text post that quotes another post.
    pub async fn create_quote_post(
        &self,
        text: &str,
        quoted_post_id: &PostId,
    ) -> crate::Result<Post> {
        let content = TextPostContent {
            text: text.to_owned(),
            quoted_post_id: Some(quoted_post_id.clone()),
            link_attachment: None,
            poll_attachment: None,
            reply_control: None,
            reply_to_id: None,
            topic_tag: None,
            allowlisted_country_codes: None,
            location_id: None,
            auto_publish_text: false,
            text_entities: None,
            text_attachment: None,
            gif_attachment: None,
            is_ghost_post: false,
            enable_reply_approvals: false,
        };
        self.create_text_post(&content).await
    }

    /// Create a media container and return its ID.
    ///
    /// Public wrapper around the internal container creation. Useful for
    /// building custom publishing flows (e.g. carousel children).
    pub async fn create_media_container(
        &self,
        media_type: &str,
        media_url: &str,
        alt_text: Option<&str>,
    ) -> crate::Result<ContainerId> {
        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        let user_id = self.user_id().await;
        if user_id.is_empty() {
            return Err(error::new_authentication_error(
                401,
                constants::ERR_EMPTY_USER_ID,
                "",
            ));
        }

        let mut params = HashMap::new();
        params.insert("media_type".into(), media_type.to_owned());

        let url_key = match media_type {
            "VIDEO" => "video_url",
            _ => "image_url",
        };
        params.insert(url_key.into(), media_url.to_owned());

        if let Some(alt) = alt_text {
            params.insert("alt_text".into(), alt.to_owned());
        }

        let id = self.create_container(params).await?;
        Ok(ContainerId::from(id.as_str()))
    }

    /// Repost an existing post.
    pub async fn repost_post(&self, post_id: &PostId) -> crate::Result<Post> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        let token = self.access_token().await;
        if token.is_empty() {
            return Err(error::new_authentication_error(
                401,
                "Access token is required",
                "",
            ));
        }

        // Use the direct repost endpoint
        let path = format!("/{}/repost", post_id);
        let resp = self.http_client.post(&path, None, &token).await?;

        #[derive(serde::Deserialize)]
        struct RepostResponse {
            id: String,
        }

        let repost_resp: RepostResponse = resp.json()?;
        let repost_id = PostId::from(repost_resp.id.as_str());
        self.get_post(&repost_id).await
    }

    /// Get the status of a media container.
    pub async fn get_container_status(
        &self,
        container_id: &ContainerId,
    ) -> crate::Result<ContainerStatus> {
        if !container_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_CONTAINER_ID,
                "",
                "container_id",
            ));
        }

        let token = self.access_token().await;
        let mut params = HashMap::new();
        params.insert("fields".into(), constants::CONTAINER_STATUS_FIELDS.into());

        let path = format!("/{}", container_id);
        let resp = self.http_client.get(&path, params, &token).await?;
        resp.json()
    }

    // ---- Private helpers ----

    /// Create a media container and return its ID.
    async fn create_container(&self, params: HashMap<String, String>) -> crate::Result<String> {
        let token = self.access_token().await;
        let user_id = self.user_id().await;
        let path = format!("/{}/threads", user_id);
        let body = RequestBody::Form(params);
        let resp = self.http_client.post(&path, Some(body), &token).await?;

        #[derive(serde::Deserialize)]
        struct ContainerResponse {
            id: String,
        }

        let container: ContainerResponse = resp.json()?;
        Ok(container.id)
    }

    /// Publish a media container and return the resulting post.
    async fn publish_container(&self, container_id: &str) -> crate::Result<Post> {
        let token = self.access_token().await;
        let user_id = self.user_id().await;
        let path = format!("/{}/threads_publish", user_id);

        let mut params = HashMap::new();
        params.insert("creation_id".into(), container_id.to_owned());

        let body = RequestBody::Form(params);
        let resp = self.http_client.post(&path, Some(body), &token).await?;
        resp.json()
    }

    /// Poll container status until it is FINISHED/PUBLISHED, ERROR, or EXPIRED.
    async fn wait_for_container_ready(&self, container_id: &ContainerId) -> crate::Result<()> {
        for attempt in 0..constants::DEFAULT_CONTAINER_POLL_MAX_ATTEMPTS {
            let status = self.get_container_status(container_id).await?;

            if status.status == constants::CONTAINER_STATUS_FINISHED
                || status.status == constants::CONTAINER_STATUS_PUBLISHED
            {
                return Ok(());
            }

            if status.status == constants::CONTAINER_STATUS_ERROR {
                let msg = status
                    .error_message
                    .unwrap_or_else(|| "Container processing failed".into());
                return Err(error::new_api_error(0, &msg, "", ""));
            }

            if status.status == constants::CONTAINER_STATUS_EXPIRED {
                return Err(error::new_api_error(
                    0,
                    "Container expired before publishing",
                    "",
                    "",
                ));
            }

            // Don't sleep after the last attempt
            if attempt < constants::DEFAULT_CONTAINER_POLL_MAX_ATTEMPTS - 1 {
                tokio::time::sleep(constants::DEFAULT_CONTAINER_POLL_INTERVAL).await;
            }
        }

        Err(error::new_api_error(
            0,
            "Container status polling timed out",
            &format!(
                "Container {} did not reach FINISHED after {} attempts",
                container_id,
                constants::DEFAULT_CONTAINER_POLL_MAX_ATTEMPTS
            ),
            "",
        ))
    }

    fn build_text_params(
        &self,
        content: &TextPostContent,
        _user_id: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("media_type".into(), constants::MEDIA_TYPE_TEXT.into());
        params.insert("text".into(), content.text.clone());

        if let Some(ref link) = content.link_attachment {
            params.insert("link_attachment".into(), link.clone());
        }
        if let Some(ref rc) = content.reply_control {
            params.insert(
                "reply_control".into(),
                serde_json::to_string(rc)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }
        if let Some(ref reply_to) = content.reply_to_id {
            params.insert("reply_to_id".into(), reply_to.to_string());
        }
        if let Some(ref topic) = content.topic_tag {
            params.insert("topic_tag".into(), topic.clone());
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            if !codes.is_empty() {
                params.insert("allowlisted_country_codes".into(), codes.join(","));
            }
        }
        if let Some(ref loc) = content.location_id {
            params.insert("location_id".into(), loc.clone());
        }
        if let Some(ref qp) = content.quoted_post_id {
            params.insert("quote_post_id".into(), qp.to_string());
        }
        if content.is_ghost_post {
            params.insert("is_ghost_post".into(), "true".into());
        }
        if content.enable_reply_approvals {
            params.insert("enable_reply_approvals".into(), "true".into());
        }
        if let Some(ref entities) = content.text_entities {
            if let Ok(json) = serde_json::to_string(entities) {
                params.insert("text_entities".into(), json);
            }
        }
        if let Some(ref attachment) = content.text_attachment {
            if let Ok(json) = serde_json::to_string(attachment) {
                params.insert("text_attachment".into(), json);
            }
        }
        if let Some(ref gif) = content.gif_attachment {
            if let Ok(json) = serde_json::to_string(gif) {
                params.insert("gif_attachment".into(), json);
            }
        }

        params
    }

    fn build_image_params(
        &self,
        content: &ImagePostContent,
        _user_id: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("media_type".into(), constants::MEDIA_TYPE_IMAGE.into());
        params.insert("image_url".into(), content.image_url.clone());

        if let Some(ref text) = content.text {
            params.insert("text".into(), text.clone());
        }
        if let Some(ref alt) = content.alt_text {
            params.insert("alt_text".into(), alt.clone());
        }
        if let Some(ref rc) = content.reply_control {
            params.insert(
                "reply_control".into(),
                serde_json::to_string(rc)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }
        if let Some(ref reply_to) = content.reply_to_id {
            params.insert("reply_to_id".into(), reply_to.to_string());
        }
        if let Some(ref topic) = content.topic_tag {
            params.insert("topic_tag".into(), topic.clone());
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            if !codes.is_empty() {
                params.insert("allowlisted_country_codes".into(), codes.join(","));
            }
        }
        if let Some(ref loc) = content.location_id {
            params.insert("location_id".into(), loc.clone());
        }
        if let Some(ref qp) = content.quoted_post_id {
            params.insert("quote_post_id".into(), qp.to_string());
        }
        if content.is_spoiler_media {
            params.insert("is_spoiler_media".into(), "true".into());
        }
        if content.enable_reply_approvals {
            params.insert("enable_reply_approvals".into(), "true".into());
        }
        if let Some(ref entities) = content.text_entities {
            if let Ok(json) = serde_json::to_string(entities) {
                params.insert("text_entities".into(), json);
            }
        }

        params
    }

    fn build_video_params(
        &self,
        content: &VideoPostContent,
        _user_id: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("media_type".into(), constants::MEDIA_TYPE_VIDEO.into());
        params.insert("video_url".into(), content.video_url.clone());

        if let Some(ref text) = content.text {
            params.insert("text".into(), text.clone());
        }
        if let Some(ref alt) = content.alt_text {
            params.insert("alt_text".into(), alt.clone());
        }
        if let Some(ref rc) = content.reply_control {
            params.insert(
                "reply_control".into(),
                serde_json::to_string(rc)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }
        if let Some(ref reply_to) = content.reply_to_id {
            params.insert("reply_to_id".into(), reply_to.to_string());
        }
        if let Some(ref topic) = content.topic_tag {
            params.insert("topic_tag".into(), topic.clone());
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            if !codes.is_empty() {
                params.insert("allowlisted_country_codes".into(), codes.join(","));
            }
        }
        if let Some(ref loc) = content.location_id {
            params.insert("location_id".into(), loc.clone());
        }
        if let Some(ref qp) = content.quoted_post_id {
            params.insert("quote_post_id".into(), qp.to_string());
        }
        if content.is_spoiler_media {
            params.insert("is_spoiler_media".into(), "true".into());
        }
        if content.enable_reply_approvals {
            params.insert("enable_reply_approvals".into(), "true".into());
        }
        if let Some(ref entities) = content.text_entities {
            if let Ok(json) = serde_json::to_string(entities) {
                params.insert("text_entities".into(), json);
            }
        }

        params
    }

    fn build_carousel_params(
        &self,
        content: &CarouselPostContent,
        _user_id: &str,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("media_type".into(), constants::MEDIA_TYPE_CAROUSEL.into());
        let children_str: Vec<String> = content.children.iter().map(|c| c.to_string()).collect();
        params.insert("children".into(), children_str.join(","));

        if let Some(ref text) = content.text {
            params.insert("text".into(), text.clone());
        }
        if let Some(ref rc) = content.reply_control {
            params.insert(
                "reply_control".into(),
                serde_json::to_string(rc)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned(),
            );
        }
        if let Some(ref reply_to) = content.reply_to_id {
            params.insert("reply_to_id".into(), reply_to.to_string());
        }
        if let Some(ref topic) = content.topic_tag {
            params.insert("topic_tag".into(), topic.clone());
        }
        if let Some(ref codes) = content.allowlisted_country_codes {
            if !codes.is_empty() {
                params.insert("allowlisted_country_codes".into(), codes.join(","));
            }
        }
        if let Some(ref loc) = content.location_id {
            params.insert("location_id".into(), loc.clone());
        }
        if let Some(ref qp) = content.quoted_post_id {
            params.insert("quote_post_id".into(), qp.to_string());
        }
        if content.is_spoiler_media {
            params.insert("is_spoiler_media".into(), "true".into());
        }
        if content.enable_reply_approvals {
            params.insert("enable_reply_approvals".into(), "true".into());
        }
        if let Some(ref entities) = content.text_entities {
            if let Ok(json) = serde_json::to_string(entities) {
                params.insert("text_entities".into(), json);
            }
        }

        params
    }
}
