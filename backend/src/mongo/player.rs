use uuid::Uuid;

use crate::{
    email::confirmation::send_confirmation_email, errors::DBoError, mongo::models::Player,
    validation::validate_all,
};

impl Player {
    /// Register a new player.\
    /// This function does several things:
    /// - Validates the arguments to ensure that they are valid.
    /// - Attempts to send a confirmation email to the provided email address.
    /// - Inserts a new document into the
    pub async fn register(username: &str, password: &str, email: &str) -> Result<Player, DBoError> {
        match validate_all(username, password, email) {
            Ok(_) => (),
            Err(info) => return Err(DBoError::InvalidPlayerInfo(info)),
        };

        send_confirmation_email(email, username).await;

        Ok(Player {
            player_id: Uuid::new_v4(),
            username: String::from(username),
            password: String::from(password),
            email: String::from(email),
        })
    }
}
