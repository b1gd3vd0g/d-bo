//! This module handles the configuration of the MongoDB database used by the application.

use std::time::Duration;

use mongodb::{
    Client, Database, IndexModel,
    bson::doc,
    options::{Collation, CollationStrength, IndexOptions},
};
use urlencoding::encode;

use crate::{
    config::environment::ENV,
    models::{Collectible, ConfirmationToken, Player},
};

/// Returns a standard case-insensitive collation, for use while creating database indices, as well
/// as performing search queries which do not rely on case.
pub fn case_insensitive_collation() -> Collation {
    Collation::builder()
        .locale("en")
        .strength(CollationStrength::Secondary)
        .build()
}

/// Verify (and if necessary, create) the following indices on the database:
///
/// - A TTL index on the `confirmation-tokens` collection, so that documents are deleted **two
///   days** after their creation
/// - A case-insensitive index on both `email` and `username` fields within `players` collection.
///
/// ### Arguments
/// - `db`: The MongoDB database
///
/// ### Panics
/// If the indices cannot be created. This indicates an unrecoverable problem with our database
/// connection and/or setup.
#[doc(hidden)]
async fn index_database(db: &Database) {
    // TODO: Fix the database indices. Create more, create them more cleanly, etc. Consider handling
    // this during repository creation instead of during database connection.
    let _conf_tokens_ttl_index = db
        .collection::<ConfirmationToken>(&ConfirmationToken::collection_name())
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
        .expect(r#"Could not create the indices on the "confirmation-tokens" collection"#);

    let _players_indexes = db
        .collection::<Player>(&Player::collection_name())
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
        .expect(r#"Could not create the indices on the "players" collection"#);
}

/// Connect to the MongoDB database housing all of the data for the D-Bo application. This function
/// should be used to configure the **repository layer** of our application, but otherwise should
/// not be used by other modules.
///
/// ### Returns
/// A MongoDB Database
///
/// ### Panics
/// If the database connection string is invalid, or if the database indices could not be created.
pub async fn database() -> Database {
    let mongo_uri = format!(
        "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority&tls=true",
        ENV.mongo_username,
        encode(&ENV.mongo_password),
        ENV.mongo_server
    );

    let mongo_client = Client::with_uri_str(mongo_uri)
        .await
        .expect("The mongo_uri string is malformed.");

    // This is what we will return from the function to be used as an axum state.
    let mongo_database = mongo_client.database(&ENV.mongo_dbname);

    // ping_database(&mongo_database).await;
    index_database(&mongo_database).await;

    mongo_database
}
