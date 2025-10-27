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
    errors::{AuthnFailureReason, DBoError},
    handlers::{
        request_bodies::{
            AccountIdentifierRequestBody, PasswordChangeRequestBody, PasswordRequestBody,
            PlayerLoginRequestBody, PlayerRegistrationRequestBody, ProposedEmailChangeRequestBody,
            UsernameChangeRequestBody,
        },
        responses::{
            AccessTokenResponse, AccountLockedResponse, AuthnFailureResponse,
            MissingDocumentResponse, PlayerUniquenessViolationResponse, SimpleMessageResponse,
        },
    },
    services::player_service::PlayerService,
};

// //////////////// //
// HELPER FUNCTIONS //
// //////////////// //

/// Logs an unexpected `DBoError`, and returns an empty `500` HTTP response.
fn unexpected_error(error: DBoError, request_name: &str) -> Response {
    eprintln!("An unexpected DBoError occurred during {}!", request_name);
    eprintln!("This should not happen!");
    eprintln!("{:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR).into_response()
}

/// Takes in an `AuthnFailureReason` variant, and returns a `401` response with an
/// `AuthnFailureResponse` body, containing the relevant code to indicate to the client exactly what
/// went wrong.
fn authentication_failure_response(reason: AuthnFailureReason) -> Response {
    let code = match reason {
        AuthnFailureReason::BadLoginCredentials => "BLC",
        AuthnFailureReason::MissingAuthenticationToken => "MAT",
        AuthnFailureReason::BadAuthenticationToken => "BAT",
        AuthnFailureReason::ExpiredAuthenticationToken => "EAT",
        AuthnFailureReason::PrematureAuthenticationToken => "PAT",
        AuthnFailureReason::BadPassword => "BPW",
        AuthnFailureReason::CookieNotSet => "CNS",
        AuthnFailureReason::NonParseableCookie => "NPC",
        AuthnFailureReason::BadCookieCredentials => "BCC",
        AuthnFailureReason::ExpiredRefreshToken => "ERT",
        AuthnFailureReason::PlayerNotFound => "PNF",
    };

    (
        StatusCode::UNAUTHORIZED,
        Json(AuthnFailureResponse::new(code)),
    )
        .into_response()
}

/// Builds an HTTP response header which sets an HTTP only cookie for the `/players/refresh` path.
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

/// Extracts the access token (if it exists) from the HTTP request headers.
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
/// - `__arg0`: The Repositories stored in the axum router's state
/// - `__arg1`: The HTTP request body
///
/// ### Returns
/// - Success:
///   - `201 CREATED` with `SafePlayerResponse` body
/// - Error:
///   - `400 BAD REQUEST`:
///     - with `InputValidationResponse` body if input fails validation
///     - with `SimpleMessageResponse` body if the `time_zone` cannot be parsed
///   - `409 CONFLICT` with `ExistingFieldViolationResponse` body
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, if the player's email cannot be
///     parsed into a Mailbox, or if an unexpected `DBoError` occurs.
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

/// Handle a request to confirm a newly registered player account.
///
/// ### Arguments
/// - `__arg0`: The Repositories stored in the axum router's state.
/// - `__arg1`: The player's id, then the confirmation token's id.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `403 FORBIDDEN` if the token is not associated with the same player id.
///   - `404 NOT FOUND` with `MissingDocumentResponse` body if either the player or token could not
///     be found.
///   - `409 CONFLICT` if the player's account has already been confirmed.
///   - `410 GONE` if the confirmation token has expired after 15 minutes.
///   - `500 INTERNAL SERVER ERROR` if a database query failed, or an unexpected `DBoError` variant
///     was returned.
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
            DBoError::PersistentTokenExpired => (StatusCode::GONE).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "account confirmation"),
        },
    }
}

/// Handle a request to **reject** a newly registered player account.
///
/// ### Arguments
/// - `__arg0`: The Repositories stored in the axum router's state.
/// - `__arg1`: The player's id, then the confirmation token's id.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `403 FORBIDDEN` if the token does not represent the same player account.
///   - `404 NOT FOUND` with `MissingDocumentResponse` if the player or token cannot be found.
///   - `409 CONFLICT` if the account has already been confirmed.
///   - `500 INTERNAL SERVER ERROR` if any database query fails, or if an unexpected `DBoError`
///     variant occurs.
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
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::MissingDocument(collection) => (
                StatusCode::NOT_FOUND,
                Json(MissingDocumentResponse::new(&collection)),
            )
                .into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "account rejection"),
        },
    }
}

/// Handle a request to resend a new account's confirmation email.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The player's id, then the confirmation token's id.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `403 FORBIDDEN` if the token does not represent the same player.
///   - `404 NOT FOUND` with a `MissingDocumentResponse` if either the player or confirmation token
///     cannot be found.
///   - `409 CONFLICT` if the player account is already confirmed.
///   - `500 INTERNAL SERVER ERROR` if an adapter fails, if the player's stored email address cannot
///     be parsed into a Mailbox, or if an unexpected `DBoError` variant occurs.
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

/// Handle a request for a player to log in.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The HTTP request body
///
/// ### Returns
/// - Success:
///   - `200 OK`, with:
///     - An `AccessTokenResponse` body providing the player with their new access JWT.
///     - An HTTP-Only `Set-Cookie` header, providing the credentials for their refresh token, used
///       exclusively for the `/players/refresh` path.
/// - Error:
///   - `401 UNAUTHORIZED` with an `AuthnFailureResponse` if the provided login credentials were not
///      correct.
///   - `403 FORBIDDEN` with an `AccountLockedResponse` body if the player's account is locked.
///   - `409 CONFLICT` if the player's account is unconfirmed.
///   - `500 INTERNAL SERVER ERROR` if:
///     - An adapter function fails.
///     - The login credentials were wrong, AND either:
///       - The player's stored email cannot be parsed into a Mailbox, OR
///       - The player's stored time zone could not be parsed.
///     - An unexpected `DBoError` variant occured.
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::MissingDocument(_) => {
                authentication_failure_response(AuthnFailureReason::BadLoginCredentials)
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

/// Handle a request to refresh a player's authentication tokens.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state
/// - `cookies`: The cookies sent with the HTTP request
///
/// ### Returns
/// - Success:
///   - `200 OK` with:
///     - An `AccessTokenResponse` body providing the player with their new access JWT.
///     - An HTTP-Only `Set-Cookie` header, providing the credentials for a fresh refresh token,
///       used exclusively for the `/players/refresh` path.
/// - Error:
///   - `401 UNAUTHORIZED` with an `AuthnFailureResponse` body if the player could not be
///     authenticated with the provided cookie credentials.
///   - `500 INTERNAL SERVER ERROR` if an adapter function failed, or if an unexpected `DBoError`
///     variant occured.
pub async fn handle_player_refresh(
    State(repos): State<Repositories>,
    cookies: CookieJar,
) -> Response {
    let token_info = match cookies.get("refresh_token") {
        Some(cookie) => cookie.value(),
        None => return authentication_failure_response(AuthnFailureReason::CookieNotSet),
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::MissingDocument(_) => {
                authentication_failure_response(AuthnFailureReason::BadCookieCredentials)
            }
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "player authentication refresh"),
        },
    }
}

/// Handle a request to delete a player's own account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state
/// - `headers`: The headers sent with the HTTP request
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `401 UNAUTHORIZED` with an `AuthnFailureResponse` body if the JWT authentication fails *or*
///     if the provided password does not match the database.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, or if an unexpected `DBoError`
///     variant occurs.
pub async fn handle_player_deletion(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<PasswordRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => {
            return authentication_failure_response(AuthnFailureReason::MissingAuthenticationToken);
        }
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "player deletion"),
        },
    }
}

/// Handle a request to change a player's username.
///
/// This function will also invalidate a player's sessions, requiring them to log in again in order
/// to use their account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `headers`: The headers sent with the HTTP request.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `400 BAD REQUEST` with a `PlayerInvalidFieldsResponse` if the new username does not pass
///     validation checks.
///   - `401 UNAUTHORIZED` with an `AuthnFailureResponse` body if JWT authentication fails *or* the
///     provided password does not match the database.
///   - `409 CONFLICT` with a `PlayerUniquenessViolationResponse` if the new username is not
///     case-insensitively unique.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, if the player's stored email
///     address cannot be parsed into a Mailbox, or if an unexpected `DBoError` variant occurs.
pub async fn handle_player_username_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<UsernameChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => {
            return authentication_failure_response(AuthnFailureReason::MissingAuthenticationToken);
        }
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::BAD_REQUEST, Json(probs)).into_response()
            }
            DBoError::UniquenessViolation(u, e) => (
                StatusCode::CONFLICT,
                Json(PlayerUniquenessViolationResponse::new(u, e)),
            )
                .into_response(),
            DBoError::InvalidEmailAddress | DBoError::AdapterError => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => unexpected_error(e, "username change"),
        },
    }
}

/// Handle a request to change a player's current password.
///
/// This function will also invalidate a player's current sessions, requiring them to log in again
/// in order to use their account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `headers`: The headers sent with the HTTP request.
/// - `__arg2`: The HTTP request body.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `400 BAD REQUEST` with a `PlayerInvalidFieldsResponse` if the provided `new_password` does
///     not pass validation checks
///   - `401 UNAUTHORIZED` if the JWT authentication fails, or if the provided `old_password` does
///     not match the database.
///   - `409 CONFLICT` if the `new_password` matches any of the player's last five used passwords.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, if the player's stored email
///     address cannot be parsed into a Mailbox, or if an unexpected `DBoError` variant occurs.
pub async fn handle_player_password_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<PasswordChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => {
            return authentication_failure_response(AuthnFailureReason::MissingAuthenticationToken);
        }
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::BAD_REQUEST, Json(probs)).into_response()
            }
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::InvalidEmailAddress | DBoError::AdapterError => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => unexpected_error(e, "change password"),
        },
    }
}

/// Handle a request to change a player's **proposed** email address.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `headers`: The headers sent with the HTTP request.
/// - `__arg2`: The HTTP request body.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `400 BAD REQUEST` with a `PlayerInvalidFieldsResponse` if the proposed email address does
///     not pass validation checks.
///   - `401 UNAUTHORIZED` if JWT authentication fails, *or* if the provided password is incorrect.
///   - `409 CONFLICT` with a `PlayerUniquenessViolationResponse` if the proposed email address is
///     not case-insensitively unique.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, if the either the player's current
///     *or* proposed email address cannot be parsed into a Mailbox, or if an unexpected `DBoError`
///     variant occurs.
pub async fn handle_player_proposed_email_change(
    State(repos): State<Repositories>,
    headers: HeaderMap,
    Json(body): Json<ProposedEmailChangeRequestBody>,
) -> Response {
    let token = match extract_access_token(headers) {
        Some(t) => t,
        None => {
            return authentication_failure_response(AuthnFailureReason::MissingAuthenticationToken);
        }
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
            DBoError::AuthenticationFailure(reason) => authentication_failure_response(reason),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::BAD_REQUEST, Json(probs)).into_response()
            }
            DBoError::UniquenessViolation(u, e) => (
                StatusCode::CONFLICT,
                Json(PlayerUniquenessViolationResponse::new(u, e)),
            )
                .into_response(),
            DBoError::InvalidEmailAddress | DBoError::AdapterError => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => unexpected_error(e, "change proposed email"),
        },
    }
}

/// Handle a request to confirm a player's proposed email address.
///
/// This function will also invalidate a player's current sessions, requiring them to log in again
/// in order to use their account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The player's id, then the confirmation token's id.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `403 FORBIDDEN` if the confirmation token does not represent the same player.
///   - `404 NOT FOUND` with a `MissingDocumentResponse` if either the player or the token could not
///     be found.
///   - `409 CONFLICT` with a:
///     - `SimpleMessageResponse` body if the player does not have a proposed email address to confirm.
///     - `PlayerUniquenessViolationResponse` body if the proposed email address is not
///       case-insensitively unique.
///     - `PlayerInvalidFieldsResponse` body if the proposed email address does not pass validation
///       checks.
///   - `410 GONE` if the confirmation token has expired.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, or if an unexpected `DBoError`
///     variant occurs.
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
            DBoError::MissingDocument(collection) => (
                StatusCode::NOT_FOUND,
                Json(MissingDocumentResponse::new(&collection)),
            )
                .into_response(),
            DBoError::PersistentTokenExpired => (StatusCode::GONE).into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::InternalConflict => (
                StatusCode::CONFLICT,
                Json(SimpleMessageResponse::new(
                    "Player does not have a proposed email address to confirm!",
                )),
            )
                .into_response(),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::CONFLICT, Json(probs)).into_response()
            }
            DBoError::UniquenessViolation(u, e) => (
                StatusCode::CONFLICT,
                Json(PlayerUniquenessViolationResponse::new(u, e)),
            )
                .into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "proposed email confirmation"),
        },
    }
}

/// Handle a request to *reject* a newly proposed email address.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The player's ID, then the undo token's ID.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `403 FORBIDDEN` if the undo token does not represent the same player account.
///   - `404 NOT FOUND` with a `MissingDocumentResponse` body if the player or undo token cannot be
///     found.
///   - `409 CONFLICT` if the player does not have a `proposed_email` field.
///   - `410 GONE` if the undo token has expired.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, or if an unexpected `DBoError`
///     variant occurs.
pub async fn handle_player_proposed_email_rejection(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
) -> Response {
    let outcome = PlayerService::reject_proposed_email(
        repos.players(),
        repos.undo_tokens(),
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
            DBoError::PersistentTokenExpired => (StatusCode::GONE).into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "proposed email rejection"),
        },
    }
}

/// Handle a request to reset a player's password, using an undo token, following an unauthorized
/// password reset.
///
/// This function will also invalidate a player's current sessions, requiring them to log in again
/// in order to use their account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The player's ID, then the undo token's ID.
/// - `__arg2`: The HTTP request body.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `400 BAD REQUEST` if the provided password does not pass validation checks.
///   - `403 FORBIDDEN` if the undo token does not represent the same player.
///   - `404 NOT FOUND` with a `MissingDocumentResponse` body if either the player or the undo token
///     cannot be found.
///   - `409 CONFLICT` if the provided password matches any of the player's previous five passwords.
///   - `410 GONE` if the undo token has expired.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, or if an unexpected `DBoError`
///     variant occurs.
pub async fn handle_player_password_change_rejection_reset(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
    Json(body): Json<PasswordRequestBody>,
) -> Response {
    let outcome = PlayerService::reset_password_following_rejecting_change(
        repos.players(),
        repos.undo_tokens(),
        &player_id,
        &token_id,
        &body.password,
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
            DBoError::PersistentTokenExpired => (StatusCode::GONE).into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::BAD_REQUEST, Json(probs)).into_response()
            }
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "reset password following change rejection"),
        },
    }
}

/// Handle a request for login assistance with a player's account.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The HTTP request body.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT` if **either**:
///     - The account was found and the assistance email was sent, or
///     - No account was found, and therefore no email was sent.
/// - Error:
///   - `400 BAD REQUEST` with a `SimpleMessageResponse` body if NO identifier is provided, or if
///     BOTH identifiers are provided.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, if the player's stored email
///     address cannot be parsed into a Mailbox, or if an unexpected `DBoError` variant occurs.
pub async fn handle_player_login_assistance_request(
    State(repos): State<Repositories>,
    Json(body): Json<AccountIdentifierRequestBody>,
) -> Response {
    let identifier = match body.validate() {
        Ok(id) => id,
        Err(message) => return (StatusCode::BAD_REQUEST, Json(message)).into_response(),
    };

    let outcome =
        PlayerService::request_login_assistance(repos.players(), repos.reset_tokens(), &identifier)
            .await;

    match outcome {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => match e {
            DBoError::MissingDocument(_) => (StatusCode::NO_CONTENT).into_response(),
            DBoError::AdapterError | DBoError::InvalidEmailAddress => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => unexpected_error(e, "request login assistance"),
        },
    }
}

/// Handle a request to reset a player's forgotten password.
///
/// ### Arguments
/// - `__arg0`: The repositories stored in the axum router's state.
/// - `__arg1`: The player's ID, then the reset token's ID.
/// - `__arg2`: The HTTP request body.
///
/// ### Returns
/// - Success:
///   - `204 NO CONTENT`
/// - Error:
///   - `400 BAD REQUEST` with a `PlayerInvalidFieldsResponse` body if the provided password does
///     not pass validation checks.
///   - `403 FORBIDDEN` if the reset token does not represent the same player account.
///   - `404 NOT FOUND` with a `MissingDocumentResponse` body if the player or reset token cannot be
///     found.
///   - `409 CONFLICT` if the new password matches any of the player's last 5 passwords.
///   - `410 GONE` if the reset token has expired.
///   - `500 INTERNAL SERVER ERROR` if an adapter function fails, or if an unexpected `DBoError`
///     variant occurs.
pub async fn handle_player_forgot_password_reset(
    State(repos): State<Repositories>,
    Path((player_id, token_id)): Path<(String, String)>,
    Json(body): Json<PasswordRequestBody>,
) -> Response {
    let outcome = PlayerService::reset_forgotten_password(
        repos.players(),
        repos.reset_tokens(),
        &player_id,
        &token_id,
        &body.password,
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
            DBoError::PersistentTokenExpired => (StatusCode::GONE).into_response(),
            DBoError::RelationalConflict => (StatusCode::FORBIDDEN).into_response(),
            DBoError::InvalidPlayerInfo(probs) => {
                (StatusCode::BAD_REQUEST, Json(probs)).into_response()
            }
            DBoError::InternalConflict => (StatusCode::CONFLICT).into_response(),
            DBoError::AdapterError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            _ => unexpected_error(e, "reset forgotten password"),
        },
    }
}
