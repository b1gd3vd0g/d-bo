pub mod confirm;
pub mod delete;
pub mod find;
pub mod register;

use uuid::Uuid;

use crate::mongo::models::Player;

impl Player {
    /// Create a new Player struct from valid player input. This function should only be used upon
    /// new player registration, as **existing** player accounts should be loaded directly from the
    /// database.
    ///
    /// **Note**: This does NOT add the player to the database! This is done with the
    /// `Player::register` function.
    ///
    /// ### Arguments
    /// - `username`: The new player's username
    /// - `hashed_password`: The new player's **hashed** password
    /// - `email`: The new player's email address
    fn new(username: &str, hashed_password: &str, email: &str) -> Self {
        Self {
            player_id: Uuid::new_v4().to_string(),
            username: String::from(username),
            password: String::from(hashed_password),
            email: String::from(email),
            confirmed: false,
        }
    }

    /// Get the player's player_id
    pub fn player_id(&self) -> String {
        self.player_id.clone()
    }

    /// Get the player's username
    pub fn username(&self) -> String {
        self.username.clone()
    }

    /// Get the player's **hashed** password
    pub fn password(&self) -> String {
        self.password.clone()
    }

    /// Get the player's email address
    pub fn email(&self) -> String {
        self.email.clone()
    }

    /// See whether or not the player's email address has been confirmed yet
    pub fn confirmed(&self) -> bool {
        self.confirmed
    }
}
