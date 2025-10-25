//! This module defines all possible errors that may occur within the application. It provides an
//! enum `DBoError` containing all possibilities.
//!
//! It has the ability to map errors from external crates (`mongodb`, `lettre`, etc.) automatically
//! to a `DBoError::AdapterError`, handling logging as well as allowing for use of the `?` operator
//! within other modules.
//!
//! Finally, it defines the type alias `DBoResult<T>`, allowing for more concise function
//! annotations.

use argon2::password_hash::Error as HashingError;
use chrono::{DateTime, Utc};
use chrono_tz::ParseError as TzParseError;
use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
use lettre::{error::Error as LettreError, transport::smtp::Error as SmtpError};
use mongodb::error::Error as MongoError;

use crate::handlers::responses::PlayerInvalidFieldsResponse;

/// The reason why authentication failed for a certain request.
#[derive(Debug)]
pub enum AuthnFailureReason {
    /// The login credentials (username/email and password) did not match our records.
    BadLoginCredentials,
    /// The authentication token was not provided (at least, not correctly).
    MissingAuthenticationToken,
    /// The authentication token could not be parsed.
    BadAuthenticationToken,
    /// The authentication token expired after 15 minutes.
    ExpiredAuthenticationToken,
    /// The authentication token was created *before* a player's sessions were invalidated.
    PrematureAuthenticationToken,
    /// The password did not match the player identified by the authentication JWT.
    BadPassword,
    /// The `refresh_token` cookie was not set.
    CookieNotSet,
    /// The `refresh_token` cookie's value could not be parsed into an **id** and a **secret**.
    NonParseableCookie,
    /// The **id** or **secret** provided in the cookie did not correspond with an existing refresh
    /// token.
    BadCookieCredentials,
    /// The refresh token was expired.
    ExpiredRefreshToken,
    /// The player represented by the token (either an authentication JWT **or** a refresh token)
    /// does not exist.
    PlayerNotFound,
}

/// Encompasses all possible errors that may occur within the D-Bo application.
#[derive(Debug)]
pub enum DBoError {
    /// The player account is currently locked.
    AccountLocked(DateTime<Utc>),
    /// An error has occurred within an adapter function.
    AdapterError,
    /// The player could not be authenticated.
    AuthenticationFailure(AuthnFailureReason),
    /// An update to a document failed due to a conflicting state within that same document. The
    /// collection name is provided in the String.
    InternalConflict,
    /// An email could not be sent to a player because their email address is **invalid**; it could
    /// not be parsed into a `lettre::message::Mailbox`. This should not happen due to our player
    /// validation functions, but is not impossible.
    InvalidEmailAddress,
    /// A user has tried to create a new account with an invalid field.
    InvalidPlayerInfo(PlayerInvalidFieldsResponse),
    /// A request has failed because a document cannot be found. The collection name is provided in
    /// the String.
    MissingDocument(String),
    /// Some kind of *persistent* token (be it an email confirmation token, undo token, etc.) is
    /// expired.
    PersistentTokenExpired,
    /// An update to a document failed due to a conflicting state with a related document.
    RelationalConflict,
    /// A time zone could not be parsed from a String! This can happen during registration, which
    /// would indicate that we are making our requests badly; or it could happen whenever we are
    /// sending an email with a timestamp to a player, indicating that we are storing bad values in
    /// our database.
    TimeZoneParseError,
    /// A user has tried to create a new account, but its unique fields are already in use.
    /// The first boolean represents a username violation, the second represents the email.
    UniquenessViolation(bool, bool),
}

impl DBoError {
    pub fn missing_document(collection: &str) -> Self {
        Self::MissingDocument(String::from(collection))
    }
}

impl From<HashingError> for DBoError {
    fn from(e: HashingError) -> Self {
        eprintln!("A HashingError has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError
    }
}

impl From<MongoError> for DBoError {
    fn from(e: MongoError) -> Self {
        eprintln!("A MongoDB driver error has occurred.");
        eprintln!("{:?}", e);
        Self::AdapterError
    }
}

impl From<SmtpError> for DBoError {
    fn from(e: SmtpError) -> Self {
        eprintln!("An SMTP error has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError
    }
}

impl From<LettreError> for DBoError {
    fn from(e: LettreError) -> Self {
        eprintln!("A Lettre error has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError
    }
}

impl From<JwtError> for DBoError {
    /// ### Returns
    /// - `AuthenticationFailure(ExpiredAuthenticationToken` if token is expired.
    /// - `AuthenticationFailure(BadAuthenticationToken` if the token is invalid.
    /// - `AdapterError` for any sort of server-side error.
    fn from(e: JwtError) -> Self {
        match e.kind() {
            JwtErrorKind::ExpiredSignature => {
                Self::AuthenticationFailure(AuthnFailureReason::ExpiredAuthenticationToken)
            }

            JwtErrorKind::InvalidToken
            | JwtErrorKind::InvalidSignature
            | JwtErrorKind::InvalidIssuer
            | JwtErrorKind::InvalidAudience
            | JwtErrorKind::InvalidSubject
            | JwtErrorKind::InvalidAlgorithm => {
                Self::AuthenticationFailure(AuthnFailureReason::BadAuthenticationToken)
            }

            _ => {
                eprintln!("An unexpected JWT error has occurred!");
                eprintln!("{:?}", e);
                Self::AdapterError
            }
        }
    }
}

impl From<TzParseError> for DBoError {
    fn from(e: TzParseError) -> Self {
        eprintln!("A Timezone Parsing Error has occurred!");
        eprintln!("This likely indicates a problem with our database!");
        eprintln!("{:?}", e);
        Self::TimeZoneParseError
    }
}

pub type DBoResult<T> = Result<T, DBoError>;
