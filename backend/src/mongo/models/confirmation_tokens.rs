use chrono::Utc;
use mongodb::Database;
use uuid::Uuid;

use crate::{
    errors::DBoError,
    mongo::models::{ConfirmationToken, Player},
};

impl ConfirmationToken {
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
            created: Utc::now(),
            used: false,
        }
    }

    fn token_id(&self) -> String {
        self.token_id.clone()
    }

    /// Determine whether a confirmation token is expired. A confirmation token is good for **15
    /// minutes** following its creation.
    pub fn expired(&self) -> bool {
        let delta_time = Utc::now() - self.created;
        let seconds_elapsed = delta_time.num_seconds();
        seconds_elapsed < 60 * 15
    }

    pub async fn insert(&self, db: &Database) -> Result<String, DBoError> {
        let player = Player::find_by_id(db, &self.player_id).await;
        match player {
            Ok(option) => match option {
                Some(_) => (),
                None => {
                    return Err(DBoError::RelationalConflict(format!(
                        "player_id {} does not correspond with an active player account.",
                        self.player_id
                    )));
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "There was an error with the MongoDB driver.",
                )));
            }
        };

        let insertion = db
            .collection::<ConfirmationToken>("confirmation-tokens")
            .insert_one(self)
            .await;

        match insertion {
            Ok(_) => Ok(self.token_id()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::ServerSideError(String::from(
                    "There was an error with the MongoDB driver.",
                )))
            }
        }
    }
}
