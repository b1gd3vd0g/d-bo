use mongodb::{Database, bson::doc};

use crate::{errors::DBoError, mongo::models::Player};

impl Player {
    pub async fn confirm(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let confirmation = db
            .collection::<Self>(&Self::collection())
            .find_one_and_update(
                doc! { "player_id": player_id },
                doc! { "$set": { "confirmed": true } },
            )
            .await;

        match confirmation {
            Ok(option) => match option {
                Some(_) => Ok(()),
                None => Err(DBoError::MissingDocument),
            },
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::mongo_driver_error())
            }
        }
    }
}
