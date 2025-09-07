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
use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
use lettre::{error::Error as LettreError, transport::smtp::Error as SmtpError};
use mongodb::error::Error as MongoError;

use crate::handlers::responses::InputValidationResponse;

/// Indicates the type of AdapterError that has occurred.
#[derive(Debug)]
pub enum AdapterKind {
    /// An error has occurred while querying the database.
    Database,
    /// An error has occurred while sending an email.
    Email,
    /// An error has occurred while hashing a password.
    Hashing,
    /// An error has occurred while encoding or decoding a JWT.
    Jwt,
}

/// Encompasses all possible errors that may occur within the D-Bo application.
#[derive(Debug)]
pub enum DBoError {
    /// An error has occurred within an adapter function.
    AdapterError(AdapterKind),
    /// The player could not be authenticated.
    AuthenticationFailure,
    /// A document cannot be updated because it does not meet the conditions required for the
    /// proposed operation
    ConditionNotMet(String),
    /// An operation which should only occur once has been reattempted.
    IdempotencyError,
    /// A user has tried to create a new account with an invalid field.
    InvalidPlayerInfo(InputValidationResponse),
    /// A provided token is invalid.
    InvalidToken,
    /// A query failed because the document that it tries to update or delete could not be found.
    MissingDocument,
    /// A **search** query returned no results.
    NoMatch,
    /// The password provided does not match the records in the database.
    PasswordMismatch,
    /// A document cannot be created, because it conflicts with the current state of the database.
    /// For example, this could happen when a confirmation token is created, but it does not
    /// correspond with a valid, active player account.
    RelationalConflict(String),
    /// Some kind of token (be it an email confirmation token, JWT, etc.) is expired.
    TokenExpired,
    /// A user has tried to create a new account, but its unique fields are already in use.
    /// The first boolean represents a username violation, the second represents the email.
    UniquenessViolation(bool, bool),
}

impl From<HashingError> for DBoError {
    fn from(e: HashingError) -> Self {
        eprintln!("A HashingError has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError(AdapterKind::Hashing)
    }
}

impl From<MongoError> for DBoError {
    fn from(e: MongoError) -> Self {
        eprintln!("A MongoDB driver error has occurred.");
        eprintln!("{:?}", e);
        Self::AdapterError(AdapterKind::Database)
    }
}

impl From<SmtpError> for DBoError {
    fn from(e: SmtpError) -> Self {
        eprintln!("An SMTP error has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError(AdapterKind::Email)
    }
}

impl From<LettreError> for DBoError {
    fn from(e: LettreError) -> Self {
        eprintln!("A Lettre error has occurred!");
        eprintln!("{:?}", e);
        Self::AdapterError(AdapterKind::Email)
    }
}

impl From<JwtError> for DBoError {
    fn from(e: JwtError) -> Self {
        match e.kind() {
            JwtErrorKind::ExpiredSignature => Self::TokenExpired,

            JwtErrorKind::InvalidToken
            | JwtErrorKind::InvalidSignature
            | JwtErrorKind::InvalidIssuer
            | JwtErrorKind::InvalidAudience
            | JwtErrorKind::InvalidSubject
            | JwtErrorKind::InvalidAlgorithm => Self::InvalidToken,

            _ => {
                eprintln!("An unexpected JWT error has occurred!");
                eprintln!("{:?}", e);
                Self::AdapterError(AdapterKind::Jwt)
            }
        }
    }
}

pub type DBoResult<T> = Result<T, DBoError>;
