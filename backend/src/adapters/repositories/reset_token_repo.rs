use crate::{adapters::repositories::Repository, errors::DBoResult, models::ResetToken};

impl Repository<ResetToken> {
    pub async fn insert(&self, token: &ResetToken) -> DBoResult<()> {
        self.collection.insert_one(token).await?;

        Ok(())
    }
}
