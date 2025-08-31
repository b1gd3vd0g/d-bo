use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::Database;

use crate::{errors::DBoError, mongo::models::Player};

pub async fn handle_resend_confirmation_email(
    State(mongo_database): State<Database>,
    Path(player_id): Path<String>,
) -> Response {
    let resend = Player::resend_confirmation_email(&mongo_database, &player_id).await;

    match resend {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => match e {
            DBoError::NoMatch | DBoError::RelationalConflict(_) => {
                (StatusCode::NOT_FOUND).into_response()
            }
            DBoError::ServerSideError(str) => {
                eprintln!("{:?}", str);
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
