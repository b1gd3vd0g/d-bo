use futures::StreamExt;
use mongodb::bson::doc;

use crate::{
    adapters::repositories::Repository,
    errors::{DBoError, DBoResult},
    models::{Collectible, Identifiable, RefreshToken},
};

impl Repository<RefreshToken> {
    /// Insert a new RefreshToken into the database. If there are more than three refresh tokens
    /// for the player, delete the oldest ones until there are only three.
    ///
    /// ### Arguments
    /// - `token`: The refresh token to insert.
    ///
    /// ### Errors
    /// - `AdapterError` if a query fails.
    pub async fn insert(&self, token: &RefreshToken) -> DBoResult<()> {
        self.collection.insert_one(token).await?;

        let tokens = self.find_player_tokens(token.player_id()).await?;

        let to_delete = if tokens.len() >= 3 {
            tokens.len() - 3
        } else {
            0
        };

        for i in 0..to_delete {
            self.delete(tokens[i].id()).await?;
        }

        Ok(())
    }

    /// Replace an existing refresh token with a new one.
    ///
    /// ### Arguments
    /// - `old_token_id`: The old token's unique identifier
    /// - `new_token`: The new token to insert
    ///
    /// ### Returns
    /// - `MissingDocument` if the old token could not be found
    /// - `AdapterError` if the query should fail
    pub async fn replace(&self, old_token_id: &str, new_token: &RefreshToken) -> DBoResult<()> {
        let option = self
            .collection
            .find_one_and_replace(doc! { "token_id": old_token_id}, new_token)
            .await?;

        if option.is_some() {
            Ok(())
        } else {
            Err(DBoError::missing_document(RefreshToken::collection_name()))
        }
    }

    /// Find all refresh tokens associated with a player account.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails, or a found document cannot be parsed into a
    ///   RefreshToken.
    async fn find_player_tokens(&self, player_id: &str) -> DBoResult<Vec<RefreshToken>> {
        let mut tokens: Vec<RefreshToken> = vec![];

        let mut cursor = self
            .collection
            .find(doc! { "player_id": player_id })
            .sort(doc! { "created": 1 })
            .await?;

        while let Some(result) = cursor.next().await {
            tokens.push(result?);
        }

        Ok(tokens)
    }
}
