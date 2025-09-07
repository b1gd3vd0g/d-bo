//! This module handles all services related to **player accounts**.

use crate::{
    adapters::{email::send_confirmation_email, repositories::Repository},
    errors::DBoResult,
    handlers::responses::SafePlayerResponse,
    models::{ConfirmationToken, Identifiable, Player},
};

pub struct PlayerService {}

impl PlayerService {
    /// Create a new player account in the database, create a new confirmation token for them to
    /// use, and send a confirmation email to the provided email address.
    ///
    /// ### Arguments
    /// - `players`: The player repository
    /// - `tokens`: The confirmation tokens repository
    /// - `username`: The requested username
    /// - `password`: The requested password
    /// - `email`: The requested email address
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo` if the username, password, or email cannot pass validation.
    /// - `UniquenessViolation` if the username or email are not case-insensitively unique.
    /// - `ServerSideError` if the email templates cannot be found.
    /// - `AdapterError(Database)` if a database query fails.
    /// - `AdapterError(Hashing)` if the password cannot be hashed (probably impossible).
    /// - `AdapterError(Smtp)` if the confirmation email could not be sent.
    pub async fn register_player(
        players: &Repository<Player>,
        tokens: &Repository<ConfirmationToken>,
        username: &str,
        password: &str,
        email: &str,
    ) -> DBoResult<SafePlayerResponse> {
        let player = Player::new(username, password, email)?;
        players.insert(&player).await?;

        let token = ConfirmationToken::new(player.id());
        tokens.insert(&token).await?;

        send_confirmation_email(email, username, token.id()).await?;

        Ok(SafePlayerResponse::from(&player))
    }
}
