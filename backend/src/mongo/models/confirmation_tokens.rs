use bson::DateTime;
use chrono::Utc;
use mongodb::{Database, bson::doc};
use uuid::Uuid;

use crate::{
    errors::DBoError,
    mongo::models::{ConfirmationToken, Player},
};

impl ConfirmationToken {
    /// Get the collection housing all confirmation tokens.
    pub fn collection() -> String {
        String::from("confirmation-tokens")
    }

    /// Create a new confirmation token struct.
    ///
    /// **Note**: This does not add the token to the database! This is done with the
    /// `ConfirmationToken::register` function.
    ///
    /// ### Arguments
    /// - `player_id`: The id of the Player this token is for.
    pub fn new(player_id: &str) -> Self {
        Self {
            token_id: Uuid::new_v4().to_string(),
            player_id: String::from(player_id),
            created: DateTime::now(),
            used: false,
        }
    }

    pub fn token_id(&self) -> String {
        self.token_id.clone()
    }

    /// Determine whether a confirmation token is expired. A confirmation token is good for **15
    /// minutes** following its creation.
    pub fn expired(&self) -> bool {
        let created = self.created.to_chrono();
        (Utc::now() - created).num_seconds() > 60 * 15
    }

    pub async fn insert(&self, db: &Database) -> Result<String, DBoError> {
        let _player = match Player::find_by_id(db, &self.player_id).await {
            Ok(_) => (),
            Err(e) => match e {
                DBoError::NoMatch => {
                    return Err(DBoError::RelationalConflict(format!(
                        "player_id {} does not correspond with an active player account.",
                        self.player_id
                    )));
                }
                _ => return Err(e),
            },
        };

        let insertion = db
            .collection::<Self>(&Self::collection())
            .insert_one(self)
            .await;

        match insertion {
            Ok(_) => Ok(self.token_id()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::mongo_driver_error())
            }
        }
    }

    pub async fn delete_by_player_id(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let deletion = db
            .collection::<Self>(&Self::collection())
            .delete_many(doc! { "player_id": player_id })
            .await;

        match deletion {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::mongo_driver_error());
            }
        }
    }

    /// Confirm a player's account by finding the associated account and marking it confirmed, and
    /// flagging the token as `used`.
    ///
    /// ### Arguments
    /// - `db`: The MongoDB database
    /// - `token_id`: The token_id to search for.
    ///
    /// ### Errors
    /// - `DBoError::NoMatch`: The token cannot be found.
    /// - `DBoError::IdempotencyError`: The player's account has already been confirmed.
    /// - `DBoError::TokenExpired`: The token was found, but it has expired after 15 minutes.
    /// - `DBoError::MissingDocument`: The associated player account has already expired.
    /// - `DBoError::ServerSideError`: A MongoDB query failed.
    pub async fn confirm(db: &Database, token_id: &str) -> Result<(), DBoError> {
        // find the token
        let token = db
            .collection::<Self>(&Self::collection())
            .find_one(doc! { "token_id": token_id })
            .await;

        let token = match token {
            Ok(option) => match option {
                Some(tok) => tok,
                None => return Err(DBoError::NoMatch),
            },
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::mongo_driver_error());
            }
        };

        if token.used {
            return Err(DBoError::IdempotencyError);
        }

        if token.expired() {
            return Err(DBoError::TokenExpired);
        }

        let _ = Player::confirm(db, &token.player_id).await?;

        let update = db
            .collection::<Self>(&Self::collection())
            .find_one_and_update(
                doc! { "token_id": token_id },
                doc! { "$set": { "used": true } },
            )
            .await;

        match update {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::mongo_driver_error())
            }
        }
    }

    /// Reject an email confirmation, and wipe all related data from the database permanently.
    ///
    /// ### Arguments
    /// - `db`: The MongoDB database
    /// - `token_id`: The token id to search for.
    ///
    /// ### Errors
    /// - `DBoError::NoMatch`: The token_id could not be found.
    /// - `DBoError::ServerSideError`: A MongoDB query has failed.
    pub async fn reject(db: &Database, token_id: &str) -> Result<(), DBoError> {
        let token = db
            .collection::<Self>(&Self::collection())
            .find_one(doc! { "token_id": token_id })
            .await;

        let token = match token {
            Ok(opt) => match opt {
                Some(tok) => tok,
                None => return Ok(()), // Token and player must have already been deleted.
            },
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::mongo_driver_error());
            }
        };

        if token.used {
            return Err(DBoError::ConditionNotMet(String::from(
                "The account has already been confirmed!",
            )));
        }

        let _player_deletion = match Player::delete(db, &token.player_id).await {
            Ok(_) => (),
            Err(e) => match e {
                DBoError::NoMatch => (),
                _ => return Err(e),
            },
        };

        let token_deletion = db
            .collection::<Self>(&Self::collection())
            .find_one_and_delete(doc! { "token_id": token_id })
            .await;

        match token_deletion {
            Ok(option) => match option {
                Some(token) => match Player::delete(db, &token.player_id).await {
                    Ok(()) => Ok(()),
                    Err(e) => match e {
                        DBoError::MissingDocument => Ok(()),
                        _ => Err(e),
                    },
                },
                None => Ok(()),
            },
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::mongo_driver_error())
            }
        }
    }
}
