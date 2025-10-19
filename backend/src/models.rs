//! This module is home to the **model layer** of the application. The model layer provides
//! structure and validation logic to all models that are stored in the database.
//!
//! Models themselves are **database independent**, meaning that they do not directly interact with
//! the database; they are basically just shapes. Actual interaction with the database is handled by
//! the repository layer.

pub mod player_validation;
pub mod submodels;

use std::{array, time::Duration as StdDuration};

use bson::{DateTime, doc};
use chrono::{Duration as ChronoDuration, Utc};
use chrono_tz::Tz;
use mongodb::{Collection, IndexModel, options::IndexOptions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    adapters::{hashing::hash_secret, mongo::case_insensitive_collation},
    errors::DBoResult,
    models::{
        player_validation::validate_all,
        submodels::{Gender, LanguagePreference, PlayerStats, UndoTokenType},
    },
};

// //////////// //
// MODEL TRAITS //
// //////////// //

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

pub trait Indexed: Send + Sync + Sized {
    async fn index(collection: &Collection<Self>);
}

/// A composite trait that is required for any database model. Any struct implementing these traits
/// will automatically receive the trait `Model`.
pub trait Model:
    Collectible + Identifiable + Indexed + Serialize + for<'de> Deserialize<'de>
{
}

impl<T> Model for T where
    T: Collectible + Identifiable + Indexed + Serialize + for<'de> Deserialize<'de>
{
}

// /////////////// //
// DATABASE MODELS //
// /////////////// //

// PLAYER
// //////

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
    /// A player's proposed email address; this value is only present if a player has **requested**
    /// to change their email address, but has not yet **verified** the new one.
    proposed_email: Option<String>,
    /// The last four passwords used by this account.
    last_passwords: [String; 4],
    /// The player's gender.
    gender: Gender,
    /// The player's preferred language.
    preferred_language: LanguagePreference,
    /// The player's preferred pronouns, specifically useful while translating to Spanish for
    /// players with `gender == Gender.Other`.
    pronoun: Gender,
    /// The player's gameplay stats.
    stats: PlayerStats,
    /// The date of the player's last **successful** login.
    last_login: DateTime,
    /// The number of consecutive failed logins.
    failed_logins: u8,
    /// The date when a player can attempt to log in again.
    locked_until: Option<DateTime>,
    /// Any access JWTs or Refresh Tokens created *before* this date will be considered invalid.
    session_valid_after: DateTime,
    /// The Time Zone identifier string (i.e. "America/Los_Angeles") for the player's preferred time
    /// zone.
    time_zone: String,
}

impl Player {
    /// Construct a new player
    ///
    /// ### Arguments
    /// - `username`: The username of the new player
    /// - `password`: The raw text password of the new player
    /// - `email`: The email address of the new player
    /// - `gender`: The player's preferred gender
    /// - `preferred_language`: The player's preferred language
    /// - `pronoun`: The player's preferred pronouns
    /// - `time_zone`: The player's preferred time zone identifier string (i.e.
    ///   "America/Los_Angeles")
    ///
    /// ### Errors
    /// - `InvalidPlayerInput` if the `username`, `password`, or `email` do not pass validation
    /// - `TimeZoneParseError` if the `time_zone` cannot be parsed
    /// - `AdapterError` if password hashing fails
    pub fn new(
        username: &str,
        password: &str,
        email: &str,
        gender: &Gender,
        preferred_language: &LanguagePreference,
        pronoun: &Gender,
        time_zone: &str,
    ) -> DBoResult<Self> {
        validate_all(username, password, email)?;

        let _tz: Tz = time_zone.parse()?;

        let now = DateTime::now();

        Ok(Self {
            player_id: Uuid::new_v4().to_string(),
            username: String::from(username),
            password: hash_secret(password)?,
            email: String::from(email),
            created: now,
            confirmed: false,
            proposed_email: None,
            last_passwords: array::from_fn(|_| String::new()),
            gender: gender.clone(),
            preferred_language: preferred_language.clone(),
            pronoun: pronoun.clone(),
            stats: PlayerStats::default(),
            last_login: now,
            failed_logins: 0,
            locked_until: None,
            session_valid_after: now,
            time_zone: String::from(time_zone),
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

    pub fn gender(&self) -> &Gender {
        &self.gender
    }

    pub fn preferred_language(&self) -> &LanguagePreference {
        &self.preferred_language
    }

    pub fn pronoun(&self) -> &Gender {
        &self.pronoun
    }

    pub fn stats(&self) -> &PlayerStats {
        &self.stats
    }

    pub fn confirmed(&self) -> bool {
        self.confirmed
    }

    pub fn failed_logins(&self) -> u8 {
        self.failed_logins
    }

    pub fn locked_until(&self) -> &Option<DateTime> {
        &self.locked_until
    }

    pub fn locked(&self) -> bool {
        if let Some(time) = self.locked_until {
            time.to_chrono() > Utc::now()
        } else {
            false
        }
    }

    pub fn valid_after(&self) -> &DateTime {
        &self.session_valid_after
    }

    pub fn last_passwords(&self) -> &[String; 4] {
        &self.last_passwords
    }

    pub fn proposed_email(&self) -> &Option<String> {
        &self.proposed_email
    }

    pub fn time_zone(&self) -> &str {
        &self.time_zone
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

impl Indexed for Player {
    /// Index a collection of Players. These indices include:
    /// - A uniqueness index on `player_id`
    /// - A case-insensitive uniqueness index on `username`
    /// - A case-insensitive uniqueness index on `email`
    /// - A conditional 2-day TTL index on `created` when `confirmed == false`
    ///
    /// ### Panics
    /// If the indices cannot be created for any reason
    async fn index(collection: &Collection<Self>) {
        collection
            .create_indexes(vec![
                IndexModel::builder()
                    .keys(doc! { Self::id_field(): 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("player-id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "username": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("username-unique-insensitive"))
                            .unique(true)
                            .collation(case_insensitive_collation())
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "email": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("email-unique-insensitive"))
                            .unique(true)
                            .collation(case_insensitive_collation())
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "created": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("created-ttl-2d-condition-unconfirmed"))
                            .expire_after(StdDuration::from_secs(60 * 60 * 24 * 2))
                            .partial_filter_expression(doc! { "confirmed": false })
                            .build(),
                    )
                    .build(),
            ])
            .await
            .expect("Failed to index the Player collection!");
    }
}

// CONFIRMATION TOKEN
// //////////////////

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

    pub fn player_id(&self) -> &str {
        &self.player_id
    }

    pub fn expired(&self) -> bool {
        Utc::now() - self.created.to_chrono() > ChronoDuration::seconds(60 * 15)
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

impl Indexed for ConfirmationToken {
    /// Index a collection of ConfirmationTokens. These indices include:
    /// - A uniqueness index on `token_id`
    /// - A uniqueness index on `player_id`
    /// - A 2-day TTL index on `created`
    ///
    /// ### Panics
    /// If the indices cannot be created for any reason
    async fn index(collection: &Collection<Self>) {
        collection
            .create_indexes(vec![
                IndexModel::builder()
                    .keys(doc! { Self::id_field(): 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("token-id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "player_id": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("player-id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "created": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("created-ttl-2d"))
                            .expire_after(StdDuration::from_secs(60 * 60 * 24 * 2))
                            .build(),
                    )
                    .build(),
            ])
            .await
            .expect("Failed to index the ConfirmationToken collection!");
    }
}

// COUNTER
// ///////

/// A document representing a counter, stored in the `counters` collection.
#[derive(Clone, Deserialize, Serialize)]
pub struct Counter {
    /// The unique identifier, indicating what statistic the counter is representing
    id: String,
    /// The amount of times a counter has been incremented
    count: u64,
}

impl Counter {
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl Collectible for Counter {
    fn collection_name() -> &'static str {
        "counters"
    }
}

impl Identifiable for Counter {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_field() -> &'static str {
        "id"
    }
}

impl Indexed for Counter {
    /// Index a collection of Counters with the following index:
    /// - A uniqueness index on `id`
    ///
    /// ### Panics
    /// If the index cannot be created for any reason
    async fn index(collection: &Collection<Self>) {
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { Self::id_field(): 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
            )
            .await
            .expect("Failed to index the Counter collection!");
    }
}

// REFRESH TOKEN
// /////////////

/// A document representing a refresh token, which can validate a player whose access token has
/// expired for up to 30 days.
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

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn revoked(&self) -> bool {
        self.revoked
    }

    pub fn expired(&self) -> bool {
        Utc::now() - self.created.to_chrono() > ChronoDuration::seconds(60 * 60 * 24 * 30)
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

impl Indexed for RefreshToken {
    /// Index a collection of RefreshTokens. The indices include:
    /// - A uniqueness index on `token_id`
    /// - A standard index on `player_id`
    /// - A 30-day TTL index on `created`
    ///
    /// ### Panics
    /// If the indices cannot be created for any reason
    async fn index(collection: &Collection<Self>) {
        collection
            .create_indexes(vec![
                IndexModel::builder()
                    .keys(doc! { Self::id_field(): 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("token-id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "player_id": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("player-id-std"))
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "created": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("created-ttl-30d"))
                            .expire_after(StdDuration::from_secs(60 * 60 * 24 * 30))
                            .build(),
                    )
                    .build(),
            ])
            .await
            .expect("Failed to index the RefreshToken collection!");
    }
}

// UNDO TOKEN
// //////////

#[derive(Clone, Deserialize, Serialize)]
pub struct UndoToken {
    token_id: String,
    player_id: String,
    function: UndoTokenType,
    created: DateTime,
}

impl UndoToken {
    pub fn new(player_id: &str, function: &UndoTokenType) -> Self {
        Self {
            token_id: Uuid::new_v4().to_string(),
            player_id: String::from(player_id),
            function: function.clone(),
            created: DateTime::now(),
        }
    }

    pub fn player_id(&self) -> &str {
        &self.player_id
    }

    pub fn function(&self) -> &UndoTokenType {
        &self.function
    }

    pub fn expired(&self) -> bool {
        Utc::now() - self.created.to_chrono() > ChronoDuration::seconds(60 * 60 * 24)
    }
}

impl Collectible for UndoToken {
    fn collection_name() -> &'static str {
        "undo-tokens"
    }
}

impl Identifiable for UndoToken {
    fn id(&self) -> &str {
        &self.token_id
    }

    fn id_field() -> &'static str {
        "token_id"
    }
}

impl Indexed for UndoToken {
    /// Index a collection of UndoTokens. The indices include:
    /// - A uniqueness index on `token_id`
    /// - A compound uniqueness index on `player_id` and `function`
    /// - A 1-day TTL index on `created`
    ///
    /// ### Panics
    /// If the indices cannot be created for any reason
    async fn index(collection: &Collection<Self>) {
        collection
            .create_indexes(vec![
                IndexModel::builder()
                    .keys(doc! { Self::id_field(): 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("token-id-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "created": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("created-1d-ttl"))
                            .expire_after(StdDuration::from_secs(60 * 60 * 24))
                            .build(),
                    )
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "player_id": 1, "function": 1 })
                    .options(
                        IndexOptions::builder()
                            .name(String::from("player-id-function-compound-unique"))
                            .unique(true)
                            .build(),
                    )
                    .build(),
            ])
            .await
            .expect("Failed to index the UndoToken collection!");
    }
}
