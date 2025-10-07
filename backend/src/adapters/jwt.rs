//! This module is an adapter over the `jsonwebtoken` crate, handling access tokens for the
//! application.

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{config::environment::ENV, errors::DBoResult};

/// A JWT payload used to authenticate a player, valid for 15 minutes.
#[derive(Deserialize, Serialize)]
pub struct AccessTokenPayload {
    /// The player_id of the represented player.
    sub: String,
    /// The timestamp for when the token is set to expire.
    exp: usize,
    /// The timestamp for when the token was issued.
    iat: usize,
}

impl AccessTokenPayload {
    /// Construct a new access token payload
    ///
    /// ### Arguments
    /// - `sub`: The player_id of the player to represent
    pub fn new(sub: &str) -> Self {
        let now = Utc::now();
        Self {
            sub: String::from(sub),
            exp: (now + Duration::minutes(15)).timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }

    /// Return the player_id represented by this token.
    pub fn sub(&self) -> &str {
        &self.sub
    }

    /// Returns true if a token was made before a specified time
    ///
    /// ### Arguments
    /// - `time`: The time to compare to.
    pub fn made_before(&self, time: &DateTime<Utc>) -> bool {
        self.iat < time.timestamp() as usize
    }
}

/// Encode an access token for a player
///
/// ### Arguments
/// - `player_id`: The player's unique identifier
///
/// ### Errors
/// - `AdapterError` if the token cannot be encoded
pub fn generate_access_token(player_id: &str) -> DBoResult<String> {
    let payload = AccessTokenPayload::new(player_id);
    Ok(encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(ENV.authn_token_secret.as_bytes()),
    )?)
}

/// Decode an access token
///
/// ### Arguments
/// - `token`: The access token
///
/// ### Returns
/// The token's payload
///
/// ### Errors
/// - `TokenExpired` if the token is expired
/// - `InvalidToken` if the token cannot be decoded because it is bad
/// - `AdapterError` if the token cannot be decoded due to a server-side error
pub fn decode_access_token(token: &str) -> DBoResult<AccessTokenPayload> {
    Ok(decode::<AccessTokenPayload>(
        token,
        &DecodingKey::from_secret(ENV.authn_token_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?
    .claims)
}
