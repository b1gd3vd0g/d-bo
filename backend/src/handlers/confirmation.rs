use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::Database;

use crate::{errors::DBoError, mongo::models::ConfirmationToken};

pub async fn handle_token_confirmation(
    State(mongo_database): State<Database>,
    Path(token_id): Path<String>,
) -> Response {
    let confirmation = ConfirmationToken::confirm(&mongo_database, &token_id).await;

    match confirmation {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => match e {
            DBoError::NoMatch => (StatusCode::NOT_FOUND).into_response(),
            DBoError::TokenExpired => (StatusCode::GONE).into_response(),
            DBoError::MissingDocument => (StatusCode::CONFLICT).into_response(),
            DBoError::IdempotencyError => (StatusCode::BAD_REQUEST).into_response(),
            DBoError::ServerSideError(str) => {
                eprintln!("{}", str);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => {
                eprintln!(
                    "An unexpected DBoError occurred during token confirmation! This should not happen!"
                );
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}

pub async fn handle_token_rejection(
    State(mongo_database): State<Database>,
    Path(token_id): Path<String>,
) -> Response {
    let rejection = ConfirmationToken::reject(&mongo_database, &token_id).await;

    match rejection {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => match e {
            DBoError::NoMatch => (StatusCode::NOT_FOUND).into_response(),
            DBoError::ServerSideError(str) => {
                eprintln!("{}", str);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            _ => {
                eprintln!(
                    "An unexpected DBoError occurred during token confirmation! This should not happen!"
                );
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        },
    }
}
