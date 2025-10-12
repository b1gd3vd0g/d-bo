//! This module defines all JSON response bodies that may be returned by the HTTP handler functions.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::{
    Identifiable, Player,
    submodels::{Gender, LanguagePreference, PlayerStats},
};

/// Returned when a player account cannot be created or modified, due to its fields violating a
/// uniqueness requirement.
#[derive(Serialize)]
pub struct PlayerUniquenessViolationResponse {
    /// The fields which violated uniqueness requirements. Its values may only be "username" and
    /// "email".
    uniqueness_violations: Vec<String>,
}

impl PlayerUniquenessViolationResponse {
    /// Construct a new PlayerUniquenessViolationResponse struct.
    ///
    /// ### Arguments
    /// - `username`: Is the username already taken?
    /// - `email`: Is the email address already taken?
    pub fn new(username: bool, email: bool) -> Self {
        let mut uniqueness_violations = vec![];

        if username {
            uniqueness_violations.push(String::from("username"));
        }

        if email {
            uniqueness_violations.push(String::from("email"));
        }

        Self {
            uniqueness_violations,
        }
    }
}

/// Contains the validation information for all input fields at once.\
///
/// **Note**: This struct is serializable as it will be returned in the HTTP response body when a
/// user provides bad input. However, it will only include the fields which **failed validation**
/// within that serialized version.
#[derive(Debug, Serialize)]
pub struct PlayerInvalidFieldsResponse {
    /// A list of problems with the username.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    username_problems: Option<Vec<String>>,
    /// A list of problems with the password.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    password_problems: Option<Vec<String>>,
    /// A list of problems with the email.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    email_problems: Option<Vec<String>>,
}

impl PlayerInvalidFieldsResponse {
    /// Construct a new PlayerInvalidFieldsResponse
    ///
    /// ### Arguments
    /// - `username_problems`: A list of problems with the username
    /// - `password_problems`: A list of problems with the password
    /// - `email_problems`: A list of problems with the email
    pub fn new(
        username_problems: Option<Vec<String>>,
        password_problems: Option<Vec<String>>,
        email_problems: Option<Vec<String>>,
    ) -> Self {
        Self {
            username_problems,
            password_problems,
            email_problems,
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

/// Return an Access Token to the player - a JWT that can be used to authenticate them for 15
/// minutes.
#[derive(Serialize)]
pub struct AccessTokenResponse {
    access_token: String,
}

impl AccessTokenResponse {
    /// Create a new AccessTokenResponse struct
    ///
    /// ### Arguments
    /// - `access_token`: The access JWT
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
    /// Create a new MissingDocumentResponse
    ///
    /// ### Arguments
    /// - `collection`: The collection from which the document is missing
    pub fn new(collection: &str) -> Self {
        Self {
            missing: String::from(collection),
        }
    }
}

/// An error response indicating that the account is locked - the player cannot log into their
/// account until the time provided.
#[derive(Serialize)]
pub struct AccountLockedResponse {
    /// The UTC DateTime indicating when the account will become unlocked again, in RFC 3339
    locked_until: String,
}

impl AccountLockedResponse {
    /// Create a new AccountLockedResponse
    ///
    /// ### Arguments
    /// - `date`: The time at which the account will become unlocked again.
    pub fn new(date: DateTime<Utc>) -> Self {
        Self {
            locked_until: date.to_rfc3339(),
        }
    }
}
