use axum::{Router, routing::post};
use mongodb::Database;

use crate::handlers::player::register::handle_player_registration;

pub fn router() -> Router<Database> {
    Router::new().route("/player", post(handle_player_registration))
}
