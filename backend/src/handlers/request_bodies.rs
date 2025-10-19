/// This module contains all the request bodies that are required in incoming HTTP requests.
use serde::Deserialize;

use crate::models::submodels::{Gender, LanguagePreference};

/// The required request body for registering a new player account.
#[derive(Deserialize)]
pub struct PlayerRegistrationRequestBody {
    /// The requested username
    username: String,
    /// The requested password
    password: String,
    /// The requested email address
    email: String,
    /// The player's preferred gender
    gender: Gender,
    /// The player's preferred language
    preferred_language: LanguagePreference,
    /// The player's chosen pronouns
    pronoun: Option<Gender>,
    /// The player's initial time zone identifier string (i.e. "America/Los_Angeles")
    time_zone: String,
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

    pub fn gender(&self) -> &Gender {
        &self.gender
    }

    pub fn preferred_language(&self) -> &LanguagePreference {
        &self.preferred_language
    }

    pub fn pronoun(&self) -> &Option<Gender> {
        &self.pronoun
    }

    pub fn time_zone(&self) -> &str {
        &self.time_zone
    }
}

#[derive(Deserialize)]
pub struct PlayerLoginRequestBody {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct PasswordRequestBody {
    pub password: String,
}

#[derive(Deserialize)]
pub struct UsernameChangeRequestBody {
    pub new_username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct PasswordChangeRequestBody {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ProposedEmailChangeRequestBody {
    pub new_email: String,
    pub password: String,
}
