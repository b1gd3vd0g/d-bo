use mongodb::Database;

use crate::{errors::DBoError, hashing::verify_password, mongo::models::Player};

impl Player {
    pub async fn login(
        db: &Database,
        username_or_email: &str,
        password: &str,
    ) -> Result<(), DBoError> {
        let player = Player::find_by_email_or_username(db, username_or_email).await?;

        match verify_password(password, &player.password()) {
            Ok(b) => match b {
                true => Ok(()),
                false => return Err(DBoError::PasswordMismatch),
            },
            Err(_) => {
                return Err(DBoError::ServerSideError(String::from(
                    "An error occurred while verifying password match.",
                )));
            }
        }
    }
}
