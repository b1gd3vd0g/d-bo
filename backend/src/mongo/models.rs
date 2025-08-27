use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// This is a counter of how many times we have tested our connection to the mongodb database.
#[derive(Deserialize, Serialize)]
pub struct PingCounter {
    pings: u32,
}
