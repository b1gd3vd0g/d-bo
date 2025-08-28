pub mod models;

use std::env;

use mongodb::{Client, Database, bson::doc};
use urlencoding::encode;

use crate::mongo::models::PingCounter;

pub async fn d_bo_database() -> Database {
    let mongo_username = env::var("MONGO_USERNAME")
        .expect(r#"Environment variable "MONGO_USERNAME" is not configured."#);
    let mongo_password = env::var("MONGO_PASSWORD")
        .expect(r#"Environment variable "MONGO_PASSWORD" is not configured."#);
    let mongo_server = env::var("MONGO_SERVER")
        .expect(r#"Environment variable "MONGO_SERVER" is not configured."#);
    let mongo_dbname = env::var("MONGO_DBNAME")
        .expect(r#"Environment variable "MONGO_DBNAME" is not configured."#);

    let mongo_uri = format!(
        "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority&tls=true",
        mongo_username,
        encode(&mongo_password),
        mongo_server
    );

    let mongo_client = Client::with_uri_str(mongo_uri)
        .await
        .expect("The mongo_uri string is malformed.");

    // This is what we will return from the function to be used as an axum state.
    let mongo_database = mongo_client.database(&mongo_dbname);

    let filter = doc! { "name": "pings" };
    let update = doc! { "$inc": { "pings": 1 } };

    // Test to make sure that the connection works.
    let _ping = mongo_database
        .collection::<PingCounter>("pings")
        .find_one_and_update(filter, update)
        .upsert(true)
        .await
        .expect("Failed to connect to the mongodb database.");

    mongo_database
}
