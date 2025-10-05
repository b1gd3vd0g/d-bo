//! This module defines all JSON response bodies that may be returned by the HTTP handler functions.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::{
    Identifiable, Player,
    submodels::{Gender, LanguagePreference, PlayerStats},
};

/// Returned when registration fails due to user input not being **case-insensitively unique**
/// within the database.
#[derive(Serialize)]
pub struct ExistingFieldViolationResponse {
    /// Indicates whether the username already exists.
    username: bool,
    /// Indicates whether the email already exists.
    email: bool,
}

impl ExistingFieldViolationResponse {
    /// Construct a new ExistingFieldViolationResponse
    ///
    /// ### Arguments
    /// - `username`: Does the username already exist?
    /// - `email`: Does the email already exist?
    pub fn new(username: bool, email: bool) -> Self {
        Self {
            username: username,
            email: email,
        }
    }
}

/// Contains the validation information for all input fields at once.\
///
/// **Note**: This struct is serializable as it will be returned in the HTTP response body when a
/// user provides bad input. However, it will only include the fields which **failed validation**
/// within that serialized version.
#[derive(Debug, Serialize)]
pub struct InputValidationResponse {
    /// A list of problems with the username.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    username: Option<Vec<String>>,
    /// A list of problems with the password.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    password: Option<Vec<String>>,
    /// A list of problems with the email.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    email: Option<Vec<String>>,
}

impl InputValidationResponse {
    /// Construct a new InputValidationResponse
    ///
    /// ### Arguments
    /// - `username`: A list of problems with the username
    /// - `password`: A list of problems with the password
    /// - `email`: A list of problems with the email
    pub fn new(
        username: Option<Vec<String>>,
        password: Option<Vec<String>>,
        email: Option<Vec<String>>,
    ) -> Self {
        Self {
            username: username,
            password: password,
            email: email,
        }
    }
}

/// Contains information related to a player account, but hides any private information that would
/// not be safe to share.
#[derive(Serialize)]
pub struct SafePlayerResponse {
    /// The player's unique identifier
    player_id: String,
    /// The player's username
    username: String,
    /// The player's email address
    email: String,
    /// The time at which the player account was created, in UTC time, converted to RFC 3339
    created: String,
    /// The player's gender
    gender: Gender,
    /// The player's preferred language
    preferred_language: LanguagePreference,
    /// The player's preferred pronouns
    pronoun: Gender,
    /// A tracker of the player's wins, losses, and dropouts
    stats: PlayerStats,
}

impl SafePlayerResponse {
    /// Construct a new SafePlayerResponse from a complete Player
    ///
    /// ### Arguments
    /// - `player`: The complete player account
    pub fn from(player: &Player) -> Self {
        Self {
            player_id: String::from(player.id()),
            username: String::from(player.username()),
            email: String::from(player.email()),
            created: player.created().to_chrono().to_rfc3339(),
            gender: player.gender().clone(),
            preferred_language: player.preferred_language().clone(),
            pronoun: player.pronoun().clone(),
            stats: player.stats().clone(),
        }
    }
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    access_token: String,
}

impl AccessTokenResponse {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: String::from(access_token),
        }
    }
}

/// An error response indicating that a document could not be found.
#[derive(Serialize)]
pub struct MissingDocumentResponse {
    /// The collection from which the missing document is absent.
    missing: String,
}

impl MissingDocumentResponse {
    pub fn new(collection: &str) -> Self {
        Self {
            missing: String::from(collection),
        }
    }
}

/// An error response indicating that the account is locked.
#[derive(Serialize)]
pub struct AccountLockedResponse {
    /// The UTC DateTime indicating when the account will become unlocked again, in RFC 3339
    locked_until: String,
}

impl AccountLockedResponse {
    pub fn new(date: DateTime<Utc>) -> Self {
        Self {
            locked_until: date.to_rfc3339(),
        }
    }
}
