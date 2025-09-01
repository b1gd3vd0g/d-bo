use argon2::{
    Argon2,
    password_hash::{
        Error as HashingError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};

/// Hash a user provided password to securely store it in the database.
///
/// **Note**: The `password` argument must have already passed validation checks before being passed
/// into this function - this makes an `Err` result very unlikely.
///
/// ### Arguments
/// - `password`: The user provided, raw text password to be hashed.
///
/// ### Returns
/// - `Ok`: The secure hash.
/// - `Err`: The error that occurred while trying to hash the password. This is very unlikely, but
///   not impossible.
pub fn hash_password(password: &str) -> Result<String, HashingError> {
    let salt = SaltString::generate(&mut OsRng);

    match Argon2::default().hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => {
            eprintln!(
                "An error has occurred while creating a new password hash. This should not happen!"
            );
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}

/// Verify that a user provided password matches a secure hash that was stored in the database.
///
/// ### Arguments
/// - `password`: The user provided, raw-text password.
/// - `hash`: The secure hash from the database.
///
/// ### Returns
/// - `Ok`: `true` if the password matches; otherwise `false`.
/// - `Err`: The error that occurred while reading the hash. This is very unlikely, and would likely
///   signify that there is data corruption within the database.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, HashingError> {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!(
                "An error has occurred while parsing a password hash from the database. This should not happen!"
            );
            eprintln!("{:?}", e);
            return Err(e);
        }
    };

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
