use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::Database;
use serde::Deserialize;

use crate::{
    email,
    errors::DBoError,
    handlers::responses::{ExistingFieldViolationResponse, MessageResponse},
    mongo::models::Player,
};

#[derive(Deserialize)]
struct PlayerRegistrationRequestBody {
    username: String,
    password: String,
    email: String,
}

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
            DBoError::NonexistentEmail => {
                (
                StatusCode::BAD_REQUEST,
                Json(MessageResponse::new(
                    "A confirmation email could not be sent, likely because the provided email address does not exist.",
                ))).into_response()
            }
            DBoError::UniquenessViolation(username, email) => {
                (StatusCode::CONFLICT, Json(ExistingFieldViolationResponse::new(username, email))).into_response()
            }
            DBoError::ServerSideError(str) => {
                eprintln!("{}", str);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(MessageResponse::server_side_error())).into_response()
            }
        },
    }
}
