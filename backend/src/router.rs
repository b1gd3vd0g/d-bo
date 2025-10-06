//! This module will handle the creation of the HTTP router, as well as configure CORS settings.

use axum::{Router, routing::post};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    adapters::repositories::Repositories,
    handlers::player_handlers::{
        handle_player_account_confirmation, handle_player_account_rejection, handle_player_login,
        handle_player_registration, handle_resend_registration_email,
    },
};

/// Return the CORS configuration for the application.
fn cors() -> CorsLayer {
    // TODO: Make the cors configuration more strict, once the router is more complete.
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Return the HTTP router which will handle all incoming requests.
pub fn router() -> Router<Repositories> {
    Router::new()
        .route("/players", post(handle_player_registration))
        .route(
            "/players/{player_id}/confirm/{token_id}",
            post(handle_player_account_confirmation)
                .delete(handle_player_account_rejection)
                .put(handle_resend_registration_email),
        )
        .route("/players/login", post(handle_player_login))
        .layer(cors())
}
