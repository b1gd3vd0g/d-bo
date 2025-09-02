mod cors;
mod email;
mod environment;
mod errors;
mod handlers;
mod hashing;
mod mongo;
mod router;
mod validation;

use std::{env, net::SocketAddr};

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use tokio::net::TcpListener;

use crate::{cors::cors, environment::ENV, router::router};

#[tokio::main]
async fn main() {
    Lazy::force(&ENV);

    let mongo_database = mongo::connect().await;
    let app = router().with_state(mongo_database).layer(cors());

    let address = SocketAddr::from(([0, 0, 0, 0], 60600));
    let listener = TcpListener::bind(address).await.unwrap();

    println!("Listening on {}", address.to_string());

    axum::serve(listener, app).await.unwrap();
}
