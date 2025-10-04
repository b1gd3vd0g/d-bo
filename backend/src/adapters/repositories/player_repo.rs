//! This module provides unique functionality for the player repository.

use mongodb::bson::doc;

use crate::{
    adapters::{mongo::case_insensitive_collation, repositories::Repository},
    errors::{DBoError, DBoResult},
    models::{Collectible, Identifiable, Player},
};

impl Repository<Player> {
    /// Find a player by their email address.
    ///
    /// ### Arguments
    /// - `email`: The email address to search for (complete and case-insensitive)
    ///
    /// ### Returns
    /// The player if it can be found
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails
    pub async fn find_by_email(&self, email: &str) -> DBoResult<Option<Player>> {
        Ok(self
            .collection
            .find_one(doc! { "email": email })
            .collation(case_insensitive_collation())
            .await?)
    }

    /// Find a player by their username.
    ///
    /// ### Arguments
    /// - `username`: The username to search for (unique and case-insensitive)
    ///
    /// ### Returns
    /// The player if it can be found
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails
    pub async fn find_by_username(&self, username: &str) -> DBoResult<Option<Player>> {
        Ok(self
            .collection
            .find_one(doc! { "username": username })
            .collation(case_insensitive_collation())
            .await?)
    }

    /// Find a player by their username *or* email address.
    ///
    /// ### Arguments
    /// - `username_or_email`: The username/email address to search for (unique and
    ///   case-insensitive)
    ///
    /// ### Returns
    /// The player, if it can be found
    ///
    /// ### Errors
    /// - `AdapterError` if the query fails
    pub async fn find_by_username_or_email(
        &self,
        username_or_email: &str,
    ) -> DBoResult<Option<Player>> {
        Ok(self
            .collection
            .find_one(doc! {
                "$or": [
                    { "username": username_or_email },
                    { "email": username_or_email }
                ]
            })
            .collation(case_insensitive_collation())
            .await?)
    }

    /// Insert a new player into the database.
    ///
    /// ### Arguments
    /// - `player`: The player to be inserted.
    ///
    /// ### Errors
    /// - `UniquenessViolation` if the player's username or email address are not case-insensitively
    ///   unique.
    /// - `AdapterError` if the query fails
    pub async fn insert(&self, player: &Player) -> DBoResult<()> {
        let existing_username = self.find_by_username(player.username()).await?.is_some();
        let existing_email = self.find_by_email(player.email()).await?.is_some();

        if existing_username || existing_email {
            Err(DBoError::UniquenessViolation(
                existing_username,
                existing_email,
            ))
        } else {
            self.collection.insert_one(player).await?;
            Ok(())
        }
    }

    /// Confirm a player account. This function will work fine if the player is already confirmed.
    ///
    /// ### Arguments
    /// `player_id`: The player's unique identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if the player cannot be found
    /// - `AdapterError` if the query fails
    pub async fn confirm(&self, player_id: &str) -> DBoResult<()> {
        let player = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id },
                doc! { "$set": { "confirmed": true } },
            )
            .await?;

        match player.modified_count {
            0 => Err(DBoError::MissingDocument(String::from(
                Player::collection_name(),
            ))),
            _ => Ok(()),
        }
    }
}
