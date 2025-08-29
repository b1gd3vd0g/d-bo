use chrono::Utc;
use uuid::Uuid;

use crate::mongo::models::ConfirmationTokenInfo;

impl ConfirmationTokenInfo {
    pub fn new(player_id: &str) -> Self {
        Self {
            token: Uuid::new_v4().to_string(),
            player_id: String::from(player_id),
            created: Utc::now(),
            used: false,
        }
    }

    /// Determine whether a confirmation token is expired. A confirmation token is good for
    pub fn expired(&self) -> bool {
        let delta_time = Utc::now() - self.created;
        let seconds_elapsed = delta_time.num_seconds();
        seconds_elapsed < 60 * 15
    }
}
