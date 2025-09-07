//! This module is home to the **model layer** of the application. The model layer provides
//! structure and validation logic to all models that are stored in the database.
//!
//! Models themselves are **database independent**, meaning that they do not directly interact with
//! the database; they are basically just shapes. Actual interaction with the database is handled by
//! the repository layer.

pub mod game;
pub mod player_validation;

use bson::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    adapters::hashing::hash_secret, errors::DBoResult, models::player_validation::validate_all,
};

/// A trait for all Models that are stored in a specific collection.
pub trait Collectible {
    /// Return the name of the collection storing the models.
    fn collection_name() -> &'static str;
}

/// A trait for all Models that can be identified by a unique field.
pub trait Identifiable {
    /// Return the unique identifier used to find a specific document in the database.
    fn id(&self) -> &str;
    /// Return the field that the identifier is stored at.
    fn id_field() -> &'static str;
}

/// A marker trait for Models which are not constrained by things like uniqueness indices.
pub trait Unconstrained {}

/// A document representing a player's account information, stored in the `players` collection.
#[derive(Clone, Deserialize, Serialize)]
pub struct Player {
    /// A unique UUID v4 to identify the player
    player_id: String,
    /// A case-insensitively unique username for the player account
    username: String,
    /// A hash of the player's password used for logging in
    password: String,
    /// A case-insensitively unique email address at which the player can be contacted
    email: String,
    /// The time at which the player account was created
    created: DateTime,
    /// An indicator for whether or not the player's email address has ever been confirmed
    confirmed: bool,
    /// An indicator for whether or not the player's **current** email address has been confirmed
    email_verified: bool,
    /// A player's proposed email address; this value is only present if a player has **requested**
    /// to change their email address, but has not yet **verified** the new one.
    proposed_email: Option<String>,
}

impl Player {
    /// Construct a new player
    ///
    /// ### Arguments
    /// - `username`: The username of the new player
    /// - `password`: The raw text password of the new player
    /// - `email`: The email address of the new player
    ///
    /// ### Errors
    /// - `InvalidPlayerInput` if the input does not pass validation
    /// - `AdapterError` if password hashing fails
    pub fn new(username: &str, password: &str, email: &str) -> DBoResult<Self> {
        validate_all(username, password, email)?;

        Ok(Self {
            player_id: Uuid::new_v4().to_string(),
            username: String::from(username),
            password: hash_secret(password)?,
            email: String::from(email),
            created: DateTime::now(),
            confirmed: false,
            email_verified: false,
            proposed_email: None,
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn created(&self) -> &DateTime {
        &self.created
    }
}

impl Collectible for Player {
    fn collection_name() -> &'static str {
        "players"
    }
}

impl Identifiable for Player {
    fn id(&self) -> &str {
        &self.player_id
    }

    fn id_field() -> &'static str {
        "player_id"
    }
}

/// A document representing an email confirmation token, stored in the `confirmation-tokens`
/// collection.
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfirmationToken {
    /// A unique UUID v4 to identify the token
    token_id: String,
    /// The `player_id` of the Player that this token represents
    player_id: String,
    /// The time at which the confirmation token was created
    created: DateTime,
    /// An indicator for whether or not this token has been used to confirm a player's account yet.
    used: bool,
}

impl ConfirmationToken {
    pub fn new(player_id: &str) -> Self {
        Self {
            token_id: Uuid::new_v4().to_string(),
            player_id: String::from(player_id),
            created: DateTime::now(),
            used: false,
        }
    }
}

impl Collectible for ConfirmationToken {
    fn collection_name() -> &'static str {
        "confirmation-tokens"
    }
}

impl Identifiable for ConfirmationToken {
    fn id(&self) -> &str {
        &self.token_id
    }
    fn id_field() -> &'static str {
        "token_id"
    }
}

impl Unconstrained for ConfirmationToken {}

/// A document representing a counter, stored in the `counters` collection.
#[derive(Clone, Deserialize, Serialize)]
pub struct Counter {
    /// The unique name, indicating what statistic the counter is representing
    name: String,
    /// The amount of times a counter has been incremented
    counter: u64,
}

impl Counter {
    pub fn counter(&self) -> u64 {
        self.counter
    }
}

impl Collectible for Counter {
    fn collection_name() -> &'static str {
        "counters"
    }
}

impl Identifiable for Counter {
    fn id(&self) -> &str {
        &self.name
    }
    fn id_field() -> &'static str {
        "name"
    }
}

/// A document representing a refresh token, which can validate a player whose access token has
/// expired for up to 7 days.
#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshToken {
    /// A unique UUID v4 to identify the token
    token_id: String,
    /// The unique identifier of the player represented by this token
    player_id: String,
    /// The hashed secret to store in the database
    secret: String,
    /// The time at which the refresh token was created
    created: DateTime,
    /// Indicates whether or not the token has been revoked
    revoked: bool,
}

impl RefreshToken {
    /// Construct a new refresh token.
    ///
    /// ### Arguments
    /// - `player_id`: The represented player's unique identifier.
    /// - `secret`: The secret, to be hashed and safely stored in the database.
    ///
    /// ### Errors
    /// - `AdapterError` if the secret could not be hashed.
    pub fn new(player_id: &str, secret: &str) -> DBoResult<Self> {
        Ok(Self {
            token_id: Uuid::new_v4().to_string(),
            player_id: String::from(player_id),
            secret: hash_secret(secret)?,
            created: DateTime::now(),
            revoked: false,
        })
    }

    pub fn player_id(&self) -> &str {
        &self.player_id
    }
}

impl Collectible for RefreshToken {
    fn collection_name() -> &'static str {
        "refresh-tokens"
    }
}

impl Identifiable for RefreshToken {
    fn id(&self) -> &str {
        &self.token_id
    }

    fn id_field() -> &'static str {
        "token_id"
    }
}

/// A composite trait that is required for any database model. Any struct implementing these traits
/// will automatically receive the trait `Model`.
pub trait Model: Collectible + Identifiable + Serialize + for<'de> Deserialize<'de> {}
impl<T> Model for T where T: Collectible + Identifiable + Serialize + for<'de> Deserialize<'de> {}
