use serde::{Deserialize, Serialize};

/// This is a counter of how many times we have tested our connection to the mongodb database.
#[derive(Deserialize, Serialize)]
pub struct PingCounter {
    pings: u32,
}

/// This is a player document in the database.
#[derive(Deserialize, Serialize)]
pub struct Player {
    pub player_id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub confirmed: bool,
}
