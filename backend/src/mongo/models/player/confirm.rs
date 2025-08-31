use mongodb::{Database, bson::doc};

use crate::{errors::DBoError, mongo::models::Player};

impl Player {
    pub async fn confirm(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let confirmation = db
            .collection::<Self>(&Self::collection())
            .find_one_and_update(
                doc! { "player_id": player_id },
                doc! { "$set": { "confirmed": true, "email_verified": true } },
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

    pub async fn verify_new_email(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let player = Player::find_by_id(db, player_id).await?;

        if player.proposed_email.is_none() {
            return Err(DBoError::ConditionNotMet(String::from(
                "The player's email address cannot be changed, because there is no proposed address to change it to.",
            )));
        }

        let update = db
            .collection::<Self>(&Self::collection())
            .find_one_and_update(
                doc! { "player_id": player_id },
                doc! {
                    "$set": {
                        "email": player.proposed_email.unwrap(),
                        "email_verified": true
                    },
                    "$unset": {
                        "proposed_email": ""
                    }
                },
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
}
