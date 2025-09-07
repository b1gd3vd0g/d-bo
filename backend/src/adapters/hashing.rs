//! This module is an adapter over the `argon2` crate, handling secret hashing and verification.

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::errors::DBoError;

/// Hash a user provided secret to securely store it in the database.
///
/// ### Arguments
/// - `secret`: The user provided, raw text secret to be hashed.
///
/// ### Returns
/// The secure hash to store in the database.
///
/// ### Errors
/// - `AdapterError(Hashing)` indicating that the provided secret cannot be hashed.
pub fn hash_secret(secret: &str) -> Result<String, DBoError> {
    let salt = SaltString::generate(&mut OsRng);

    Ok(Argon2::default()
        .hash_password(secret.as_bytes(), &salt)?
        .to_string())
}

/// Verify that a user provided secret matches a secure hash that was stored in the database.
///
/// ### Arguments
/// - `secret`: The user provided, raw-text secret.
/// - `hash`: The secure hash from the database.
///
/// ### Returns
/// A boolean indicating whether or not the provided secret matches the hash.
///
/// ### Errors
/// - `AdapterError(Hashing)` indicating that the provided hash could not be parsed. This could
///   indicate a fatal error within our database!
pub fn verify_secret(secret: &str, hash: &str) -> Result<bool, DBoError> {
    let parsed_hash = PasswordHash::new(hash)?;

    Ok(
        match Argon2::default().verify_password(secret.as_bytes(), &parsed_hash) {
            Ok(()) => true,
            Err(_) => false,
        },
    )
}
