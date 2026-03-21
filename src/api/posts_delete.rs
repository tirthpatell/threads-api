use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::PostId;

/// Response from the delete post endpoint.
#[derive(Debug, serde::Deserialize)]
struct DeleteResponse {
    /// Whether the delete was successful. `None` when the field is absent.
    #[serde(default)]
    success: Option<bool>,
    /// The deleted post's ID.
    #[serde(default)]
    deleted_id: Option<String>,
}

impl Client {
    /// Delete a post by ID. Returns the deleted post ID.
    pub async fn delete_post(&self, post_id: &PostId) -> crate::Result<String> {
        if !post_id.is_valid() {
            return Err(error::new_validation_error(
                0,
                constants::ERR_EMPTY_POST_ID,
                "",
                "post_id",
            ));
        }

        let token = self.access_token().await;
        let path = format!("/{}", post_id);
        let resp = self.http_client.delete(&path, &token).await?;

        if let Ok(del_resp) = serde_json::from_slice::<DeleteResponse>(&resp.body) {
            if del_resp.success == Some(false) {
                return Err(error::new_api_error(
                    0,
                    "Delete request returned success=false",
                    "",
                    "",
                ));
            }
            if let Some(id) = del_resp.deleted_id {
                return Ok(id);
            }
        }

        Ok(post_id.to_string())
    }
}
