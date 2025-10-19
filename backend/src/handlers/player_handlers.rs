//! This module provides all HTTP handler functions related to player accounts.

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use crate::{
    adapters::repositories::Repositories,
    config::environment::ENV,
    errors::DBoError,
    handlers::{
        request_bodies::{
            PasswordChangeRequestBody, PasswordRequestBody, PlayerLoginRequestBody,
            PlayerRegistrationRequestBody, ProposedEmailChangeRequestBody,
            UsernameChangeRequestBody,
        },
        responses::{
            AccessTokenResponse, AccountLockedResponse, MissingDocumentResponse,
            PlayerUniquenessViolationResponse, SimpleMessageResponse,
        },
    },
    services::player_service::PlayerService,
};

// //////////////// //
// HELPER FUNCTIONS //
// //////////////// //

fn unexpected_error(error: DBoError, request_name: &str) -> Response {
    eprintln!("An unexpected DBoError occurred during {}!", request_name);
    eprintln!("This should not happen!");
    eprintln!("{:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR).into_response()
}

fn build_refresh_token_header(id: &str, secret: &str) -> HeaderMap {
    let cookie_value = format!("{}:{}", id, secret);
    let cookie = Cookie::build(("refresh_token", cookie_value))
        .http_only(true)
        .secure(ENV.secure())
        .same_site(SameSite::Strict)
        .path("/players/refresh")
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    headers
}

fn extract_access_token(headers: HeaderMap) -> Option<String> {
    let header = match headers.get("Authorization") {
        Some(h) => h.to_str(),
        None => return None,
    };

    let value = match header {
        Ok(v) => v.to_string(),
        Err(_) => return None,
    };

    let token = value.strip_prefix("Bearer ");

    match token {
        Some(t) => Some(t.to_string()),
        None => None,
    }
}

// //////// //
// HANDLERS //
// //////// //

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
        body.time_zone(),
    )
    .await;

    match outcome {
        Ok(info) => (StatusCode::CREATED, Json(info)).into_response(),
        Err(e) => match e {
            DBoError::InvalidPlayerInfo(info) => {
                (StatusCode::BAD_REQUEST, Json(info)).into_response()
            }
            DBoError::TimeZoneParseError => (
                StatusCode::BAD_REQUEST,
                Json(SimpleMessageResponse::new(
                    "The provided time_zone could not be parsed!",
                )),
            )
                .into_response(),
            DBoError::UniquenessViolation(username, email) => (
                StatusCode::CONFLICT,
                Json(PlayerUniquenessViolationResponse::new(username, email)),
            )
                .into_response(),
            DBoError::AdapterError | DBoError::InvalidEmailAddress => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
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
            let headers =
                build_refresh_token_header(&info.refresh_token_id, &info.refresh_token_secret);

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
            DBoError::AdapterError
            | DBoError::InvalidEmailAddress
            | DBoError::TimeZoneParseError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "player login"),
        },
    }
}

pub async fn handle_resend_registration_email(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
) -> Response {
    let outcome = PlayerService::resend_registration_email(
        repos.players(),
        repos.confirmation_tokens(),
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
            DBoError::AdapterError | DBoError::InvalidEmailAddress => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => unexpected_error(e, "resend registration email"),
        },
    }
}

pub async fn handle_player_refresh(
    State(repos): State<Repositories>,
    cookies: CookieJar,
) -> Response {
    let token_info = match cookies.get("refresh_token") {
        Some(cookie) => cookie.value(),
        None => return (StatusCode::UNAUTHORIZED).into_response(),
    };

    let output =
        PlayerService::refresh_authn_tokens(repos.players(), repos.refresh_tokens(), token_info)
            .await;

    match output {
        Ok(info) => {
            let headers =
                build_refresh_token_header(&info.refresh_token_id, &info.refresh_token_secret);

            (
                StatusCode::OK,
                headers,
                Json(AccessTokenResponse::new(&info.access_token)),
            )
                .into_response()
        }
        Err(e) => match e {
            DBoError::InvalidToken
            | DBoError::AuthenticationFailure
            | DBoError::MissingDocument(_) => (StatusCode::UNAUTHORIZED).into_response(),
            DBoError::TokenExpired => (StatusCode::GONE).into_response(),
            DBoError::InternalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "player authentication refresh"),
        },
    }
}

pub async fn handle_player_deletion(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<PasswordRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST).into_response(),
    };

    let outcome = PlayerService::delete_player_account(
        repos.players(),
        repos.counters(),
        &token,
        &body.password,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            _ => unexpected_error(e, "player deletion"),
        },
    }
}

pub async fn handle_player_username_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<UsernameChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST).into_response(),
    };

    let outcome = PlayerService::change_username(
        repos.players(),
        repos.refresh_tokens(),
        &token,
        &body.password,
        &body.new_username,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            _ => unexpected_error(e, "username change"),
        },
    }
}

pub async fn handle_player_password_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<PasswordChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST).into_response(),
    };

    let outcome = PlayerService::change_password(
        repos.players(),
        repos.undo_tokens(),
        &token,
        &body.old_password,
        &body.new_password,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            _ => unexpected_error(e, "change password"),
        },
    }
}

pub async fn handle_player_proposed_email_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<ProposedEmailChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => return (StatusCode::BAD_REQUEST).into_response(),
    };

    let outcome = PlayerService::change_proposed_email(
        repos.players(),
        repos.confirmation_tokens(),
        repos.undo_tokens(),
        &token,
        &body.password,
        &body.new_email,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            _ => unexpected_error(e, "change proposed email"),
        },
    }
}

pub async fn handle_player_proposed_email_confirmation(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
) -> Response {
    let outcome = PlayerService::confirm_proposed_email(
        repos.players(),
        repos.confirmation_tokens(),
        repos.undo_tokens(),
        &player_id,
        &token_id,
    )
    .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            _ => unexpected_error(e, "proposed email confirmation"),
        },
    }
}
