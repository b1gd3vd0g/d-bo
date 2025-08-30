use crate::validation::InputValidation;

/// Encompasses all possible errors that may occur within the D-Bo application.
#[derive(Debug)]
pub enum DBoError {
    /// A user has tried to create a new account with an invalid field.
    InvalidPlayerInfo(InputValidation),
    /// A query failed because the document that it tries to update or delete could not be found.
    MissingDocument,
    /// An email could not be sent to the player, **very likely** (but not guaranteed) because the
    /// provided email address does not exist.
    NonexistentEmail,
    /// A document cannot be created, because it conflicts with the current state of the database.
    /// For example, this could happen when a confirmation token is created, but it does not
    /// correspond with a valid, active player account.
    RelationalConflict(String),
    /// A server-side error has occurred. These are very unlikely, but not impossible. The String
    /// should contain a brief description of what went wrong. These strings should NOT be passed to
    /// the user within the HTTP request body, but instead should be logged using the eprintln!
    /// macro.
    ServerSideError(String),
    /// A user has tried to create a new account, but its unique fields are already in use.
    /// The first boolean represents a username violation, the second represents the email.
    UniquenessViolation(bool, bool),
}

impl DBoError {
    pub fn mongo_driver_error() -> Self {
        Self::ServerSideError(String::from("There was an error with the MongoDB driver."))
    }
}
