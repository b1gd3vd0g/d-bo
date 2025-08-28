use crate::validation::InputValidation;

/// Encompasses all possible errors that may occur within the D-Bo application.
pub enum DBoError {
    /// A user has tried to create a new account with an invalid field.
    InvalidPlayerInfo(InputValidation),
    /// A user has tried to create a new account, but its unique fields are already in use.
    /// The first boolean represents a username violation, the second represents the email.
    UniquenessViolation(bool, bool),
}
