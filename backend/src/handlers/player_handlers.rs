//! This module provides all HTTP handler functions related to player accounts.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    adapters::repositories::Repositories,
    errors::DBoError,
    handlers::{
        request_bodies::PlayerRegistrationRequestBody, responses::ExistingFieldViolationResponse,
    },
    services::player_service::PlayerService,
};

/// Handle a request to create a new player account.
///
/// ### Arguments
/// - `repos`: The Repositories stored in the axum router's state
/// - `body`: The HTTP request body
///
/// ### Returns
/// - Success
///   - `201 CREATED` with a `SafePlayerResponse` body
/// - Error
///   - `400 BAD REQUEST`
///     - with `InputValidationResponse` body if input fails validation
///     - with plaintext message if JSON body is malformed
///   - `409 CONFLICT` with an `ExistingFieldViolationResponse` body
///   - `422 UNPROCESSABLE ENTITY` with plaintext message if request body is missing fields
///   - `500 INTERNAL SERVER ERROR` if an HTTP adapter failed
pub async fn handle_player_registration(
    State(repos): State<Repositories>,
    Json(body): Json<PlayerRegistrationRequestBody>,
) -> Response {
    let outcome = PlayerService::register_player(
        repos.players(),
        repos.confirmation_tokens(),
        body.username(),
        body.password(),
        body.email(),
    )
    .await;

    match outcome {
        Ok(info) => (StatusCode::CREATED, Json(info)).into_response(),
        Err(e) => match e {
            DBoError::InvalidPlayerInfo(info) => {
                (StatusCode::BAD_REQUEST, Json(info)).into_response()
            }
            DBoError::UniquenessViolation(username, email) => (
                StatusCode::CONFLICT,
                Json(ExistingFieldViolationResponse::new(username, email)),
            )
                .into_response(),
            DBoError::AdapterError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => {
                eprintln!("An unexpected DBoError occurred during player registration!");
                eprintln!("This should not happen!");
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}
