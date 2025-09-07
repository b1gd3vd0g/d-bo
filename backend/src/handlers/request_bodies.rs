/// This module contains all the request bodies that are required in incoming HTTP requests.
use serde::Deserialize;

/// The required request body for registering a new player account.
#[derive(Deserialize)]
pub struct PlayerRegistrationRequestBody {
    /// The requested username
    username: String,
    /// The requested password
    password: String,
    /// The requested email address
    email: String,
}

impl PlayerRegistrationRequestBody {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

pub struct PlayerLoginRequestBody {
    pub username_or_email: String,
    pub password: String,
}
