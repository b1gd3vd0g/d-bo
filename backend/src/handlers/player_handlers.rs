//! This module provides all HTTP handler functions related to player accounts.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::{
    adapters::repositories::Repositories,
    config::environment::ENV,
    errors::DBoError,
    handlers::{
        request_bodies::{PlayerLoginRequestBody, PlayerRegistrationRequestBody},
        responses::{AccessTokenResponse, ExistingFieldViolationResponse},
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
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => {
                eprintln!("An unexpected DBoError occurred during player registration!");
                eprintln!("This should not happen!");
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}

pub async fn handle_player_login(
    State(repos): State<Repositories>,
    Json(body): Json<PlayerLoginRequestBody>,
) -> Response {
    let outcome = PlayerService::login(
        repos.players(),
        repos.refresh_tokens(),
        &body.username_or_email,
        &body.password,
    )
    .await;

    match outcome {
        Ok(info) => {
            let cookie_value = format!("{}:{}", info.refresh_token_id, info.refresh_token_secret);
            let cookie = Cookie::build(("refresh_token", cookie_value))
                .http_only(true)
                .secure(ENV.secure())
                .same_site(SameSite::Strict)
                .path("/players/refresh")
                .build();

            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
            (
                StatusCode::OK,
                headers,
                Json(AccessTokenResponse::new(&info.access_token)),
            )
                .into_response()
        }
        Err(e) => match e {
            DBoError::AuthenticationFailure => (StatusCode::UNAUTHORIZED).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => {
                eprintln!("An unexpected DBoError occurred during player login!");
                eprintln!("This should not happen!");
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}
