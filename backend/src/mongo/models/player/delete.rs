use mongodb::{Database, bson::doc};

use crate::{errors::DBoError, mongo::models::Player};

impl Player {
    pub async fn delete(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let deletion = db
            .collection::<Self>("players")
            .find_one_and_delete(doc! { "player_id": player_id })
            .await;

        match deletion {
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
