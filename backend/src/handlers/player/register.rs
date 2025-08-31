use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::Database;
use serde::Deserialize;

use crate::{
    errors::DBoError, handlers::responses::ExistingFieldViolationResponse, mongo::models::Player,
};

/// This is the required format for the HTTP request body for player registration.
#[derive(Deserialize)]
pub struct PlayerRegistrationRequestBody {
    username: String,
    password: String,
    email: String,
}

/// The HTTP handler for registering a new player.
///
/// ### Arguments
/// - `mongo_database`: The MongoDB database
/// - `body`: The HTTP request body
///
/// ### Returns
/// An HTTP response to be sent back to the user.
///
/// Possible response codes include:
/// - `201`: Success!
/// - `400`: One of the following has occured:
///   - One or more fields in the response body are invalid.
///   - The confirmation email could not be sent because the provided email does not exist.
/// - `409`: Either the username or email already exist in the database.
/// - `422`: The request body could not be deserialized into the required struct.
///   - (This one is returned automatically by axum)
/// - `500`: A server-side error has occurred.
pub async fn handle_player_registration(
    State(mongo_database): State<Database>,
    Json(body): Json<PlayerRegistrationRequestBody>,
) -> Response {
    let player =
        Player::register(&mongo_database, &body.username, &body.password, &body.email).await;

    match player {
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
            DBoError::ServerSideError(str) => {
                eprintln!("{}", str);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => {
                eprintln!(
                    "An unexpected DBoError occurred during player registration! This should not happen!"
                );
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}
