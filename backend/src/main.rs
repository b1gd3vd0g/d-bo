mod cors;
mod email;
mod errors;
mod handlers;
mod hashing;
mod mongo;
mod router;
mod validation;

use std::{env, net::SocketAddr};

use dotenvy::dotenv;
use tokio::net::TcpListener;

use crate::{cors::cors, mongo::d_bo_database, router::router};

#[tokio::main]
async fn main() {
    // Configure the environment, if necessary.
    match env::var("STAGE") {
        Ok(_) => (), // The stage, and presumably all other variables, should already be set.
        Err(_) => {
            // The stage is not set externally. This means our stage is development.
            dotenv().ok();
        }
    };

    let mongo_database = d_bo_database().await;
    let app = router().with_state(mongo_database).layer(cors());

    let address = SocketAddr::from(([0, 0, 0, 0], 60600));
    let listener = TcpListener::bind(address).await.unwrap();

    println!("Listening on {}", address.to_string());

    axum::serve(listener, app).await.unwrap();
}
