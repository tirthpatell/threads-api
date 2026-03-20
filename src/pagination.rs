use crate::client::Client;
use crate::types::ids::{PostId, UserId};
use crate::types::pagination::{PostsOptions, RepliesOptions};
use crate::types::post::{Post, PostsResponse, RepliesResponse};
use crate::types::search::SearchOptions;

// ---------------------------------------------------------------------------
// Helper: extract next cursor from a Paging struct
// ---------------------------------------------------------------------------

fn next_cursor(paging: &crate::types::pagination::Paging) -> Option<String> {
    paging
        .cursors
        .as_ref()
        .and_then(|c| c.after.clone())
        .or_else(|| paging.after.clone())
}

// ---------------------------------------------------------------------------
// PostIterator
// ---------------------------------------------------------------------------

/// Iterator for paginating through a user's posts.
pub struct PostIterator<'a> {
    client: &'a Client,
    user_id: UserId,
    options: PostsOptions,
    cursor: Option<String>,
    done: bool,
}

impl<'a> PostIterator<'a> {
    /// Create a new post iterator.
    pub fn new(
        client: &'a Client,
        user_id: UserId,
        options: Option<PostsOptions>,
    ) -> Self {
        Self {
            client,
            user_id,
            options: options.unwrap_or_default(),
            cursor: None,
            done: false,
        }
    }

    /// Fetch the next page. Returns `None` when exhausted.
    pub async fn next(&mut self) -> crate::Result<Option<PostsResponse>> {
        if self.done {
            return Ok(None);
        }

        let mut opts = self.options.clone();
        if let Some(ref c) = self.cursor {
            opts.after = Some(c.clone());
        }

        let resp = self
            .client
            .get_user_posts(&self.user_id, Some(&opts))
            .await?;

        if let Some(c) = next_cursor(&resp.paging) {
            self.cursor = Some(c);
        } else {
            self.done = true;
        }

        if resp.data.is_empty() {
            self.done = true;
        }

        Ok(Some(resp))
    }

    /// Returns `true` if there are more pages.
    pub fn has_next(&self) -> bool {
        !self.done
    }

    /// Reset to the first page.
    pub fn reset(&mut self) {
        self.cursor = None;
        self.done = false;
    }

    /// Collect all remaining pages into a single `Vec<Post>`.
    pub async fn collect_all(&mut self) -> crate::Result<Vec<Post>> {
        let mut all = Vec::new();
        while self.has_next() {
            if let Some(resp) = self.next().await? {
                all.extend(resp.data);
            }
        }
        Ok(all)
    }
}

// ---------------------------------------------------------------------------
// ReplyIterator
// ---------------------------------------------------------------------------

/// Iterator for paginating through replies to a post.
pub struct ReplyIterator<'a> {
    client: &'a Client,
    post_id: PostId,
    options: RepliesOptions,
    cursor: Option<String>,
    done: bool,
}

impl<'a> ReplyIterator<'a> {
    pub fn new(
        client: &'a Client,
        post_id: PostId,
        options: Option<RepliesOptions>,
    ) -> Self {
        Self {
            client,
            post_id,
            options: options.unwrap_or_default(),
            cursor: None,
            done: false,
        }
    }

    pub async fn next(&mut self) -> crate::Result<Option<RepliesResponse>> {
        if self.done {
            return Ok(None);
        }

        let mut opts = self.options.clone();
        if let Some(ref c) = self.cursor {
            opts.after = Some(c.clone());
        }

        let resp = self
            .client
            .get_replies(&self.post_id, Some(&opts))
            .await?;

        if let Some(c) = next_cursor(&resp.paging) {
            self.cursor = Some(c);
        } else {
            self.done = true;
        }

        if resp.data.is_empty() {
            self.done = true;
        }

        Ok(Some(resp))
    }

    pub fn has_next(&self) -> bool {
        !self.done
    }

    pub fn reset(&mut self) {
        self.cursor = None;
        self.done = false;
    }

    pub async fn collect_all(&mut self) -> crate::Result<Vec<Post>> {
        let mut all = Vec::new();
        while self.has_next() {
            if let Some(resp) = self.next().await? {
                all.extend(resp.data);
            }
        }
        Ok(all)
    }
}

// ---------------------------------------------------------------------------
// SearchIterator
// ---------------------------------------------------------------------------

/// Iterator for paginating through search results.
pub struct SearchIterator<'a> {
    client: &'a Client,
    query: String,
    options: SearchOptions,
    cursor: Option<String>,
    done: bool,
}

impl<'a> SearchIterator<'a> {
    pub fn new(
        client: &'a Client,
        query: impl Into<String>,
        options: Option<SearchOptions>,
    ) -> Self {
        Self {
            client,
            query: query.into(),
            options: options.unwrap_or_default(),
            cursor: None,
            done: false,
        }
    }

    pub async fn next(&mut self) -> crate::Result<Option<PostsResponse>> {
        if self.done {
            return Ok(None);
        }

        let mut opts = self.options.clone();
        if let Some(ref c) = self.cursor {
            opts.after = Some(c.clone());
        }

        let resp = self
            .client
            .keyword_search(&self.query, Some(&opts))
            .await?;

        if let Some(c) = next_cursor(&resp.paging) {
            self.cursor = Some(c);
        } else {
            self.done = true;
        }

        if resp.data.is_empty() {
            self.done = true;
        }

        Ok(Some(resp))
    }

    pub fn has_next(&self) -> bool {
        !self.done
    }

    pub fn reset(&mut self) {
        self.cursor = None;
        self.done = false;
    }

    pub async fn collect_all(&mut self) -> crate::Result<Vec<Post>> {
        let mut all = Vec::new();
        while self.has_next() {
            if let Some(resp) = self.next().await? {
                all.extend(resp.data);
            }
        }
        Ok(all)
    }
}
