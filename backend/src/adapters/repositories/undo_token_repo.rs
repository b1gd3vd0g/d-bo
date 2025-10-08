use bson::doc;

use crate::{
    adapters::repositories::Repository,
    errors::DBoResult,
    models::{UndoToken, submodels::UndoTokenType},
};

impl Repository<UndoToken> {
    /// Insert a new Undo Token into the database, replacing any other one that may exist with the
    /// same player ID and function
    ///
    /// ### Arguments
    /// - `token`: The Undo Token to insert
    ///
    /// ### Errors
    /// - `AdapterError` if the query should fail
    pub async fn insert(&self, token: &UndoToken) -> DBoResult<()> {
        self.collection
            .find_one_and_replace(
                doc! { "player_id": token.player_id(), "function": token.function().to_string() },
                token,
            )
            .upsert(true)
            .await?;

        Ok(())
    }

    /// Delete all tokens belonging to a specific player which serve a specific function.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    /// - `function`: The function of the undo token
    ///
    /// ### Errors
    /// - `AdapterError` if the query should fail
    pub async fn delete_by_player_and_func(
        &self,
        player_id: &str,
        function: &UndoTokenType,
    ) -> DBoResult<()> {
        self.collection
            .delete_many(doc! {
                "player_id": player_id,
                "function": function.to_string()
            })
            .await?;

        Ok(())
    }
}
