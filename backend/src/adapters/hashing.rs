//! This module is an adapter over the `argon2` crate, handling password hashing and verification.

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::errors::DBoError;

/// Hash a user provided password to securely store it in the database.
///
/// **Note**: The `password` argument must have already passed validation checks before being passed
/// into this function - this makes an `Err` result very unlikely.
///
/// ### Arguments
/// - `password`: The user provided, raw text password to be hashed.
///
/// ### Returns
/// The secure hash to store in the database.
///
/// ### Errors
/// - `AdapterError(Hashing)` indicating that the provided password cannot be hashed.
pub fn hash_password(password: &str) -> Result<String, DBoError> {
    let salt = SaltString::generate(&mut OsRng);

    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Verify that a user provided password matches a secure hash that was stored in the database.
///
/// ### Arguments
/// - `password`: The user provided, raw-text password.
/// - `hash`: The secure hash from the database.
///
/// ### Returns
/// A boolean indicating whether or not the provided password matches the hash.
///
/// ### Errors
/// - `AdapterError(Hashing)` indicating that the provided hash could not be parsed. This could
///   indicate a fatal error within our database!
pub fn verify_password(password: &str, hash: &str) -> Result<bool, DBoError> {
    let parsed_hash = PasswordHash::new(hash)?;

    Ok(
        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => true,
            Err(_) => false,
        },
    )
}
