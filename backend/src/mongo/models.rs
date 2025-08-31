pub mod confirmation_tokens;
pub mod player;

use bson::DateTime;
use serde::{Deserialize, Serialize};

/// A document representing the number of `pings` we have made to the database, stored in the
/// `pings` collection. This is essentially a counter of how many times we have started the app and
/// successfully connected to the database.
#[derive(Deserialize, Serialize)]
pub struct PingCounter {
    pings: u32,
}

/// A document representing a player's account information, stored in the `players` collection.
#[derive(Deserialize, Serialize)]
pub struct Player {
    player_id: String,
    username: String,
    password: String,
    email: String,
    confirmed: bool,
}

/// A document representing an email confirmation token, stored in the `confirmation-tokens`
/// collection.
#[derive(Serialize, Deserialize)]
pub struct ConfirmationToken {
    token_id: String,
    player_id: String,
    created: DateTime,
}
