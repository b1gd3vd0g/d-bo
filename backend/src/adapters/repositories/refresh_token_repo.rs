use futures::StreamExt;
use mongodb::bson::doc;

use crate::{
    adapters::repositories::Repository,
    errors::DBoResult,
    models::{Identifiable, RefreshToken},
};

impl Repository<RefreshToken> {
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
