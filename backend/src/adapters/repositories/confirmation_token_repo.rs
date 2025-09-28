//! This module provides unique functionality for the confirmation token repository.

use bson::doc;

use crate::{adapters::repositories::Repository, errors::DBoResult, models::ConfirmationToken};

impl Repository<ConfirmationToken> {
    /// Insert a new email confirmation token into the repository. This will replace any
    /// confirmation tokens which already exist for the provided `player_id`; there should only ever
    /// be one token per player in the database at a time - requesting a new one will delete any
    /// older ones.
    ///
    /// ### Arguments
    /// - `token`: The confirmation token to insert into the database.
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails.
    pub async fn insert(&self, token: &ConfirmationToken) -> DBoResult<()> {
        self.collection
            .find_one_and_replace(doc! { "player_id": token.player_id() }, token)
            .upsert(true)
            .await?;
        Ok(())
    }
}
