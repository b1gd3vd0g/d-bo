//! **D-Bo** is a web application allowing for users to play a familiar card game with each other in
//! real time. This crate contains the entire backend for that application.
//!
//! **This crate is under active development!** Keep checking back to see further updates.
//!
//! The crate exposes a REST API, handling stateless requests such as account creation and the like.
//! Said API is currently **stable**, but is far from complete.
//!
//! In the future, this crate will also expose WebSocket functionality to allow for real-time
//! player-to-player interaction.

mod adapters;
mod config;
mod errors;
mod handlers;
mod models;
mod router;
mod services;

use std::net::SocketAddr;

use once_cell::sync::Lazy;
use tokio::net::TcpListener;

use crate::{
    adapters::repositories::{Repositories, counter_id::CounterId},
    config::{assets::ASSETS, environment::ENV},
    router::router,
};

/// Initialize lazy variables, create Repositories struct to be used as a state by the axum router,
/// ping the database to ensure a stable connection, and create the axum router to listen for
/// requests on port 60600.
#[tokio::main]
async fn main() {
    Lazy::force(&ENV);
    Lazy::force(&ASSETS);

    let repositories = Repositories::new().await;

    repositories
        .counters()
        .increment_counter(CounterId::Pings)
        .await
        .expect("Failed to ping the MongoDB database.");

    let app = router().with_state(repositories);

    let address = SocketAddr::from(([0, 0, 0, 0], 60600));
    let listener = TcpListener::bind(address).await.unwrap();

    println!("Listening on {}", address.to_string());

    axum::serve(listener, app).await.unwrap();
}
