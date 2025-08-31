use axum::{Router, routing::post};
use mongodb::Database;

use crate::handlers::{
    confirmation::{handle_token_confirmation, handle_token_rejection},
    player::{
        register::handle_player_registration, resend_confirmation::handle_resend_confirmation_email,
    },
};

pub fn router() -> Router<Database> {
    Router::new()
        .route("/players", post(handle_player_registration))
        .route(
            "/confirmations/{token_id}",
            post(handle_token_confirmation).delete(handle_token_rejection),
        )
        .route(
            "/players/{player_id}/resend",
            post(handle_resend_confirmation_email),
        )
}
