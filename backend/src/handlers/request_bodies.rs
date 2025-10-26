/// This module contains all the request bodies that are required in incoming HTTP requests.
use serde::Deserialize;

use crate::{
    handlers::responses::SimpleMessageResponse,
    models::submodels::{Gender, LanguagePreference},
};

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

#[derive(Deserialize)]
pub struct AccountIdentifierRequestBody {
    username: Option<String>,
    email: Option<String>,
}

pub enum AccountIdentifier {
    Username(String),
    Email(String),
}

impl AccountIdentifierRequestBody {
    /// Validate the shape of an AccountIdentifierRequestBody. The request body must include
    /// exactly ONE identifier!
    ///
    /// ### Returns
    /// The Account Identifier, which tells the type AND the value of the identifier to use.
    ///
    /// ### Errors
    /// Provides a SimpleMessageResponse, indicating why the request was bad, which can be returned
    /// to the client directly as a 400 response.
    pub fn validate(&self) -> Result<AccountIdentifier, SimpleMessageResponse> {
        if let Some(un) = &self.username
            && self.email.is_none()
        {
            Ok(AccountIdentifier::Username(un.clone()))
        } else if let Some(em) = &self.email
            && self.username.is_none()
        {
            Ok(AccountIdentifier::Email(em.clone()))
        } else if self.username.is_none() && self.email.is_none() {
            Err(SimpleMessageResponse::new(
                "No account identifier was provided!",
            ))
        } else {
            Err(SimpleMessageResponse::new(
                "Only ONE (1) identifier should be provided!",
            ))
        }
    }
}
