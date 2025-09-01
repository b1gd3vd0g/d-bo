pub mod models;

use std::{env, time::Duration};

use mongodb::{
    Client, Database, IndexModel,
    bson::doc,
    options::{Collation, CollationStrength, IndexOptions},
};
use urlencoding::encode;

use crate::mongo::models::{ConfirmationToken, PingCounter, Player};

/// Returns a standard case-insensitive collation for username operations.
///
/// Use this when creating indexes or performing queries that should ignore case differences.
pub fn case_insensitive_collation() -> Collation {
    Collation::builder()
        .locale("en")
        .strength(CollationStrength::Secondary)
        .build()
}

/// "Ping" the database, to ensure that the connection can actually be made. This essentially keeps
/// track of how many times the application has been restarted (and connected successfully).
///
/// ### Arguments
/// - `db`: The MongoDB database to ping.
///
/// ### Panics
/// If the database query fails. This indicates an unrecoverable problem with our database
/// connection.
async fn ping_database(db: &Database) {
    let filter = doc! { "name": "pings" };
    let update = doc! { "$inc": { "pings": 1 } };

    // Test to make sure that the connection works.
    let _ping = db
        .collection::<PingCounter>("pings")
        .find_one_and_update(filter, update)
        .upsert(true)
        .await
        .expect("Failed to connect to the mongodb database.");
}

/// Verify (and if necessary, create) the following indexes on the database:
///
/// - A TTL index on the `confirmation-tokens` collection, so that documents are deleted **two
///   days** after their creation
/// - A case-insensitive index on both `email` and `username` fields within `players` collection.
///
/// ### Arguments
/// - `db`: The MongoDB database
///
/// ### Panics
/// If the indexes cannot be created. This indicates an unrecoverable problem with our database
/// connection and/or setup.
async fn index_database(db: &Database) {
    let _conf_tokens_ttl_index = db
        .collection::<ConfirmationToken>(&ConfirmationToken::collection())
        .create_index(
            IndexModel::builder()
                .keys(doc! { "created": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Some(Duration::from_secs(2 * 24 * 60 * 60)))
                        .name(String::from("confirmation-token-ttl-index"))
                        .build(),
                )
                .build(),
        )
        .await
        .expect(r#"Could not create the indexes on the "confirmation-tokens" collection"#);

    let _players_indexes = db
        .collection::<Player>(&Player::collection())
        .create_indexes(vec![
            IndexModel::builder()
                .keys(doc! { "username": 1 })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .collation(case_insensitive_collation())
                        .name(String::from("player-case-insensitive-username-index"))
                        .build(),
                )
                .build(),
            IndexModel::builder()
                .keys(doc! { "email": 1 })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .collation(case_insensitive_collation())
                        .name(String::from("player-case-insensitive-email-index"))
                        .build(),
                )
                .build(),
            IndexModel::builder()
                .keys(doc! { "created": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Duration::from_secs(60 * 60 * 24 * 2))
                        .partial_filter_expression(doc! { "confirmed": false })
                        .name(String::from("unconfirmed-player-ttl-index"))
                        .build(),
                )
                .build(),
        ])
        .await
        .expect(r#"Could not create the indexes on the "players" collection"#);
}

/// Connect to the MongoDB database housing all of the data for the D-Bo application.
///
/// ### Returns
/// A MongoDB Database, to use as a State in the axum router.
///
/// ### Panics
/// If the database connection string is invalid, or if the database cannot be pinged, or if the
/// database indexes could not be created.
pub async fn connect() -> Database {
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
    ping_database(&mongo_database).await;
    index_database(&mongo_database).await;

    mongo_database
}
