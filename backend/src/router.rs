use axum::Router;
use mongodb::Database;

pub fn router() -> Router<Database> {
    Router::new()
}
