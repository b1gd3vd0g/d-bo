//! This module provides all HTTP handler functions related to player accounts.

use axum::{
    Json,
    extract::{Path, State},
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
        responses::{
            AccessTokenResponse, AccountLockedResponse, ExistingFieldViolationResponse,
            MissingDocumentResponse,
        },
    },
    services::player_service::PlayerService,
};

fn unexpected_error(error: DBoError, request_name: &str) -> Response {
    eprintln!("An unexpected DBoError occurred during {}!", request_name);
    eprintln!("This should not happen!");
    eprintln!("{:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR).into_response()
}

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
        repos.counters(),
        body.username(),
        body.password(),
        body.email(),
        body.gender(),
        body.preferred_language(),
        body.pronoun(),
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
            _ => unexpected_error(e, "player registration"),
        },
    }
}

pub async fn handle_player_account_confirmation(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
) -> Response {
    let outcome = PlayerService::confirm_player_account(
        repos.players(),
        repos.confirmation_tokens(),
        repos.counters(),
        &player_id,
        &token_id,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            DBoError::MissingDocument(collection) => (
                StatusCode::NOT_FOUND,
                Json(MissingDocumentResponse::new(&collection)),
            )
                .into_response(),
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::TokenExpired => (StatusCode::GONE).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "account confirmation"),
        },
    }
}

pub async fn handle_player_account_rejection(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
) -> Response {
    let outcome = PlayerService::reject_player_account(
        repos.players(),
        repos.confirmation_tokens(),
        repos.counters(),
        &player_id,
        &token_id,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            DBoError::InternalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::MissingDocument(_) => (StatusCode::NOT_FOUND).into_response(),
            DBoError::RelationalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "account rejection"),
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
        repos.counters(),
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
            DBoError::AuthenticationFailure | DBoError::MissingDocument(_) => {
                (StatusCode::UNAUTHORIZED).into_response()
            }
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::AccountLocked(time) => (
                StatusCode::FORBIDDEN,
                Json(AccountLockedResponse::new(time)),
            )
                .into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "player login"),
        },
    }
}
