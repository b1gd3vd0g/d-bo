//! This module will handle the creation of the HTTP router, as well as configure CORS settings.

use axum::{
    Router,
    routing::{post, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    adapters::repositories::Repositories,
    handlers::player_handlers::{
        handle_player_account_confirmation, handle_player_account_rejection,
        handle_player_deletion, handle_player_login, handle_player_password_change,
        handle_player_proposed_email_change, handle_player_proposed_email_confirmation,
        handle_player_refresh, handle_player_registration, handle_player_username_change,
        handle_resend_registration_email,
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
        .route(
            "/players",
            post(handle_player_registration).delete(handle_player_deletion),
        )
        .route(
            "/players/{player_id}/confirm/{token_id}",
            post(handle_player_account_confirmation)
                .delete(handle_player_account_rejection)
                .put(handle_resend_registration_email),
        )
        .route("/players/login", post(handle_player_login))
        .route("/players/refresh", post(handle_player_refresh))
        .route(
            "/players/change/password",
            put(handle_player_password_change),
        )
        .route(
            "/players/change/username",
            put(handle_player_username_change),
        )
        .route(
            "/players/change/proposed-email",
            put(handle_player_proposed_email_change),
        )
        .route(
            "/players/{player_id}/confirm-proposed-email/{token_id}",
            put(handle_player_proposed_email_confirmation),
        )
        .layer(cors())
}
