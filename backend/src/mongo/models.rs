use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// This is a counter of how many times we have tested our connection to the mongodb database.
#[derive(Deserialize, Serialize)]
pub struct PingCounter {
    pings: u32,
}

/// This is a player document in the database.
#[derive(Deserialize, Serialize)]
pub struct Player {
    player_id: Uuid,
    username: String,
    password: String,
    email: String,
}
