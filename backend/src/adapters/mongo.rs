//! This module handles the configuration of the MongoDB database used by the application.

use mongodb::{
    Client, Database,
    options::{Collation, CollationStrength},
};
use urlencoding::encode;

use crate::config::environment::ENV;

/// Returns a standard case-insensitive collation, for use while creating database indices, as well
/// as performing search queries which do not rely on case.
pub fn case_insensitive_collation() -> Collation {
    Collation::builder()
        .locale("en")
        .strength(CollationStrength::Secondary)
        .build()
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

    mongo_database
}
