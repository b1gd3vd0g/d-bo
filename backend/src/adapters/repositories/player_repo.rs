//! This module provides unique functionality for the player repository.

use bson::DateTime;
use chrono::{Duration, Utc};
use mongodb::{bson::doc, options::ReturnDocument};

use crate::{
    adapters::{
        hashing::{hash_secret, verify_secret},
        jwt::decode_access_token,
        mongo::case_insensitive_collation,
        repositories::Repository,
    },
    errors::{DBoError, DBoResult},
    handlers::responses::InputValidationResponse,
    models::{
        Collectible, Identifiable, Player,
        player_validation::{validate_email, validate_password, validate_username},
    },
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

    /// Find a player via a JWT access token.
    ///
    /// ### Arguments
    /// - `jwt`: The JWT
    ///
    /// ### Errors
    /// - `TokenExpired` if the JWT is expired
    /// - `TokenPremature` if the JWT was made before player sessions were invalidated
    /// - `InvalidToken` if the token is bad
    /// - `MissingDocument` if the player cannot be found
    /// - `AdapterError` if the database query fails, or if the token cannot be decoded due to a
    ///   server-side error
    pub async fn find_by_token(&self, jwt: &str) -> DBoResult<Player> {
        let payload = decode_access_token(jwt)?;

        let player = match self.find_by_id(payload.sub()).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        if payload.made_before(&player.valid_after().to_chrono()) {
            return Err(DBoError::TokenPremature);
        }

        Ok(player)
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
        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id },
                doc! { "$set": { "confirmed": true } },
            )
            .await?;

        match update.matched_count {
            0 => Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }

    /// Increment the number of failed logins on a player account. If the number of failed logins
    /// then meets or exceeds 5, it will lock the account for 15 minutes * `failed_logins - 4`.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    ///
    /// ### Returns
    /// The date until which the account is locked
    ///
    /// ### Errors
    /// - `MissingDocument` if the account cannot be found
    /// - `AdapterError` if any query should fail
    pub async fn increment_failed_logins(&self, player_id: &str) -> DBoResult<Option<DateTime>> {
        let player = match self.find_by_id(player_id).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::missing_document(Player::collection_name()));
            }
        };

        let failed_logins = player.failed_logins() + 1;
        let lockout_end = if failed_logins < 5 {
            None
        } else {
            let lockout_time = Duration::minutes(15) * (failed_logins as i32 - 4);
            Some(DateTime::from_chrono(Utc::now() + lockout_time))
        };

        self.collection
            .find_one_and_update(
                doc! { Player::id_field(): player_id },
                doc! { "$set": {
                    "failed_logins": failed_logins as i32,
                    "locked_until": lockout_end
                } },
            )
            .return_document(ReturnDocument::After)
            .await?;

        Ok(lockout_end)
    }

    /// Record a successful login in the database, resetting the `failed_logins` field to `0` and
    /// `locked_until` back to `None`.
    ///
    /// ### Arguments
    /// - `player_id`: The unique identifier of the player
    ///
    /// ### Errors
    /// - `MissingDocument` if the player cannot be found
    /// - `AccountLocked` if the account is currently locked
    /// - `AdapterError` if a query fails
    pub async fn record_successful_login(&self, player_id: &str) -> DBoResult<()> {
        let player = match self.find_by_id(player_id).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::missing_document(Player::collection_name()));
            }
        };

        if player.locked() {
            return Err(DBoError::AccountLocked(
                player.locked_until().unwrap().to_chrono(),
            ));
        }

        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id },
                doc! { "$set": {
                    "last_login": DateTime::now(),
                    "failed_logins": 0,
                    "locked_until": None::<DateTime>
                } },
            )
            .await?;

        match update.matched_count {
            0 => Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }

    /// Update a player's username in the database. Ensure that the new username is valid, and that
    /// it is case-insensitively unique. Update the player's username, and invalidate their access
    /// tokens by setting their "session_valid_after" field.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    /// - `value`: The new email address
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo` if the username does not pass validation checks
    /// - `UniquenessViolation` if the username is already taken
    /// - `MissingDocument` if the player cannot be found
    /// - `AdapterError` if any database query should fail
    pub async fn update_username(&self, player_id: &str, value: &str) -> DBoResult<()> {
        let probs = validate_username(value);
        if probs.is_some() {
            return Err(DBoError::InvalidPlayerInfo(InputValidationResponse::new(
                probs, None, None,
            )));
        }

        let existing_player = self.find_by_username(value).await?;

        if existing_player.is_some() {
            return Err(DBoError::UniquenessViolation(true, false));
        }

        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id},
                doc! { "$set": {
                   "username": value,
                   "session_valid_after": DateTime::now()
                } },
            )
            .await?;

        match update.matched_count {
            0 => Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }

    /// Update a player's proposed email address. Validate the new value and ensure that it is
    /// case-insensitively unique. Update the "proposed_email" field in the player document.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    /// - `value`: The new proposed email address
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo` if the email address cannot be validated
    /// - `UniquenessViolation` if the email address is already in use
    /// - `MissingDocument` if the player cannot be found
    /// - `AdapterError` if any database query should fail
    pub async fn update_proposed_email(&self, player_id: &str, value: &str) -> DBoResult<()> {
        let probs = validate_email(value);
        if probs.is_some() {
            return Err(DBoError::InvalidPlayerInfo(InputValidationResponse::new(
                None, None, probs,
            )));
        }

        if self.find_by_email(value).await?.is_some() {
            return Err(DBoError::UniquenessViolation(false, true));
        }

        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id},
                doc! { "$set": { "proposed_email": value } },
            )
            .await?;

        match update.matched_count {
            0 => Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }

    /// Confirm a player's proposed email address. Find the player by id, and ensure that they have
    /// a proposed email address. Validate that email address, and ensure that it is
    /// case-insensitively unique. Update the players email to be their proposed email, and reset
    /// their proposed email to None. Invalidate a player's access tokens by changing the
    /// "session_valid_after" field.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if the player cannot be found
    /// - `InternalConflict` if the player does not have a proposed email address
    /// - `InvalidPlayerInfo` if the email address cannot be validated
    /// - `UniquenessViolation` if the email address is already in use
    /// - `AdapterError` if a database query should fail
    pub async fn confirm_proposed_email(&self, player_id: &str) -> DBoResult<()> {
        let player = match self.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let proposed = match player.proposed_email() {
            Some(p) => p,
            None => return Err(DBoError::InternalConflict),
        };

        let probs = validate_email(proposed);

        if probs.is_some() {
            return Err(DBoError::InvalidPlayerInfo(InputValidationResponse::new(
                None, None, probs,
            )));
        }

        if self.find_by_email(&proposed).await?.is_some() {
            return Err(DBoError::UniquenessViolation(false, true));
        }

        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id },
                doc! { "$set": {
                    "email": player.proposed_email(),
                    "proposed_email": None::<String>,
                    "session_valid_after": DateTime::now()
                } },
            )
            .await?;

        match update.matched_count {
            0 => return Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }

    /// Update a player's current password. Ensure that the password is valid. Find the player by
    /// their id. Ensure that the new password does not match any of their last five passwords. Push
    /// all their last passwords back in the array, freeing up the last one again; replace the 0
    /// index with their current password. Hash their new password. Update their "password" field to
    /// the hash. Invalidate the player's access tokens by changing their "session_valid_after"
    /// field.
    ///
    /// ### Arguments
    /// - `player_id`: The player's unique identifier
    /// - `value`: The new password to save.
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo` if the password is invalid.
    /// - `MissingDocument` if the player cannot be found.
    /// - `InternalConflict` if the new password matches any of the last five used.
    /// - `AdapterError` if any database query should fail, or if any of their previous password
    ///   hashes cannot be parsed, or if their current password cannot be hashed.
    pub async fn update_password(&self, player_id: &str, value: &str) -> DBoResult<()> {
        let probs = validate_password(value);
        if probs.is_some() {
            return Err(DBoError::InvalidPlayerInfo(InputValidationResponse::new(
                None, probs, None,
            )));
        }

        let player = match self.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        if verify_secret(value, player.password())? {
            return Err(DBoError::InternalConflict);
        }

        for hash in player.last_passwords() {
            if verify_secret(value, hash)? {
                return Err(DBoError::InternalConflict);
            }
        }

        let mut records = player.last_passwords().clone().to_vec();

        for i in (1..4).rev() {
            records[i] = records[i - 1].clone();
        }

        records[0] = String::from(player.password());

        let hash = hash_secret(value)?;

        let update = self
            .collection
            .update_one(
                doc! { Player::id_field(): player_id },
                doc! { "$set": {
                    "password": &hash,
                    "last_passwords": &records,
                    "session_valid_after": DateTime::now()
                } },
            )
            .await?;

        match update.matched_count {
            0 => Err(DBoError::missing_document(Player::collection_name())),
            _ => Ok(()),
        }
    }
}
