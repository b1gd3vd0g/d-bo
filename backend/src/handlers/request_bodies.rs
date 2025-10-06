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
