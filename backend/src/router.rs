use axum::{Router, routing::post};
use mongodb::Database;

use crate::handlers::{
    confirmation::{handle_token_confirmation, handle_token_rejection},
    player::register::handle_player_registration,
};

pub fn router() -> Router<Database> {
    Router::new()
        .route("/player", post(handle_player_registration))
        .route(
            "/confirmations/{token_id}",
            post(handle_token_confirmation).delete(handle_token_rejection),
        )
}
