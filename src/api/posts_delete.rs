use crate::client::Client;
use crate::constants;
use crate::error;
use crate::types::PostId;

impl Client {
    /// Delete a post by ID.
    pub async fn delete_post(&self, post_id: &PostId) -> crate::Result<()> {
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
        self.http_client.delete(&path, &token).await?;
        Ok(())
    }
}
