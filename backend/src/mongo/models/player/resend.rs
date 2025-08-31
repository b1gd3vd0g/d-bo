use mongodb::Database;

use crate::{
    email::confirmation::send_confirmation_email,
    errors::DBoError,
    mongo::models::{ConfirmationToken, Player},
};

impl Player {
    /// Resend the confirmation email to the player. This happens when a player tries to confirm
    /// their email address, but finds the token to be expired.
    ///
    /// This function will:
    /// - Find the associated player account (if it still exists).
    /// - Delete any existing tokens associated with that account.
    /// - Insert a new token into the database.
    /// - Resend the confirmation email to the player.
    pub async fn resend_confirmation_email(db: &Database, player_id: &str) -> Result<(), DBoError> {
        let player = Player::find_by_id(db, player_id).await?;

        let _token_deletion = ConfirmationToken::delete_by_player_id(db, player_id).await?;

        let new_token = ConfirmationToken::new(player_id);

        let _insertion = match new_token.insert(db).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        match send_confirmation_email(&player.email, &player.username, &new_token.token_id()).await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "The email could not be sent due to a server-side smtp error.",
                )));
            }
        }
    }
}
