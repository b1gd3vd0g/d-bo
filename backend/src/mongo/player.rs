use lettre::transport::smtp::response::{Category, Detail, Severity};
use mongodb::{Database, bson::doc, error::Error as MongoError};
use uuid::Uuid;

use crate::{
    email::confirmation::send_confirmation_email, errors::DBoError, hashing::hash_password,
    mongo::models::Player, validation::validate_all,
};

/// This struct reflects a Player object, but obscures away any sensitive data that may be dangerous
/// to expose to any malicious users.
pub struct SafePlayerInfo {
    username: String,
    email: String,
    player_id: String,
    confirmed: bool,
}

impl Player {
    /// Converts a Player into a SafePlayerInfo struct, obscuring any sensitive data.
    fn safe(&self) -> SafePlayerInfo {
        SafePlayerInfo {
            username: self.username.clone(),
            email: self.email.clone(),
            player_id: self.player_id.clone(),
            confirmed: self.confirmed,
        }
    }

    /// Find a **single** player account by case-insensitively searching for their **entire** email
    /// address.
    ///
    /// ### Arguments
    /// - `db`: The MongoDB database
    /// - `email`: The email address to search for
    ///
    /// ### Returns
    /// - `Ok<Some>`: An existing user was found.
    /// - `Ok<None>`: An existing user was not found, but the search was successful.
    /// - `Err`: The search failed due to some server-side error.
    async fn find_by_email(db: &Database, email: &str) -> Result<Option<Player>, MongoError> {
        db.collection::<Player>("players")
            .find_one(doc! { "email": { "$regex": format!("^{}$", email), "$options": "i" } })
            .await
    }

    /// Find a **single** player account by case-insensitively searching for their **entire**
    /// username.
    ///
    /// ### Arguments
    /// - `db`: The MongoDB database
    /// - `username`: The username to search for
    ///
    /// ### Returns
    /// - `Ok<Some>`: An existing user was found.
    /// - `Ok<None>`: An existing user was not found, but the search was successful.
    /// - `Err`: The search failed due to some server-side error.
    async fn find_by_username(db: &Database, username: &str) -> Result<Option<Player>, MongoError> {
        db.collection::<Player>("players")
            .find_one(doc! { "username": { "$regex": format!("^{}$", username), "$options": "i" } })
            .await
    }

    /// Register a new player.
    ///
    /// This function does several things:
    /// - Validates the arguments to ensure that the input is valid for the field.
    /// - Checks the database to make sure that the username and/or email are indeed
    ///   **case-insensitively unique**.
    /// - Attempts to send a confirmation email to the provided email address.
    /// - Inserts a new document into the database.
    /// - Return the **safe** player information (not including the hashed password).
    ///
    /// ### Arguments
    /// - `db`: The MongoDB database
    /// - `username`: The user-provided username to register
    /// - `password`: The user-provided password to register
    /// - `email`: The user-provided email address to register
    ///
    /// ### Returns
    /// - `Ok`: The **safe** player information of the newly registered player.
    /// - `Err`: The error that occurred while trying to register the new player.
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo`: The provided input did not pass validation checks.
    /// - `UniquenessViolation`: Either the username or email already exist in the database.
    /// - `NonexistentEmail`: The email could not be sent because the provided address does not
    ///   exist.
    /// - `ServerSideError`: One of the following must be true:
    ///   - The database could not be accessed at some point during the process.
    ///   - The password could not be hashed (super unlikely).
    ///   - The email could not be sent due to some server-side error
    pub async fn register(
        db: &Database,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<SafePlayerInfo, DBoError> {
        match validate_all(username, password, email) {
            Ok(_) => (),
            Err(info) => return Err(DBoError::InvalidPlayerInfo(info)),
        };

        let existing_username = match Self::find_by_username(db, username).await {
            Ok(option) => match option {
                Some(_) => true,
                None => false,
            },
            Err(e) => {
                // MongoDB Driver threw an error!
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "There was an error with the MongoDB driver.",
                )));
            }
        };

        let existing_email = match Self::find_by_email(db, email).await {
            Ok(option) => match option {
                Some(_) => true,
                None => false,
            },
            Err(e) => {
                // MongoDB Driver threw an error!
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "There was an error with the MongoDB driver.",
                )));
            }
        };

        if existing_username || existing_email {
            return Err(DBoError::UniquenessViolation(
                existing_username,
                existing_email,
            ));
        }

        let hashed_password = match hash_password(password) {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "A HashingError occurred while trying to hash the provided password!",
                )));
            }
        };

        match send_confirmation_email(email, username).await {
            Ok(_) => (),
            Err(e) => {
                if let Some(code) = e.status() {
                    if code.severity == Severity::PermanentNegativeCompletion
                        && code.category == Category::MailSystem
                    {
                        return Err(DBoError::NonexistentEmail);
                    }
                }
                eprintln!("{:?}", e);
                return Err(DBoError::ServerSideError(String::from(
                    "The email could not be sent due to a server-side smtp error.",
                )));
            }
        };

        let new_player = Player {
            username: String::from(username),
            password: hashed_password,
            email: String::from(email),
            player_id: Uuid::new_v4().to_string(),
            confirmed: false,
        };

        let insertion = db
            .collection::<Player>("players")
            .insert_one(&new_player)
            .await;

        match insertion {
            Ok(_) => Ok(new_player.safe()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(DBoError::ServerSideError(String::from(
                    "There was an error with the MongoDB driver.",
                )))
            }
        }
    }
}
