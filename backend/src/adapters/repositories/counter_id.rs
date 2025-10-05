//! This module provides an enum containing all possible IDs of Counters stored in the database, and
//! implements ToString in order to ensure safe handling of valid counters within the application.

/// An enum storing all types of Counters the app keeps track of.
pub enum CounterId {
    /// "pings": Keeps track of app startups and initial database connections.
    Pings,
    /// "accounts_registered": Keeps track of accounts registered successfully.
    AccountsRegistered,
    /// "accounts_confirmed": Keeps track of accounts successfully confirmed after registration.
    AccountsConfirmed,
    /// "accounts_rejected": Keeps track of accounts rejected following initial registration.
    AccountsRejected,
    /// "logins": Keeps track of successful logins
    Logins,
    /// "failed_logins": Keeps track of failed login attempts
    FailedLogins,
}

impl ToString for CounterId {
    /// Return the `id` field of the specific Counter.
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Pings => "pings",
            Self::AccountsRegistered => "accounts_registered",
            Self::AccountsConfirmed => "accounts_confirmed",
            Self::AccountsRejected => "accounts_rejected",
            Self::Logins => "logins",
            Self::FailedLogins => "failed_logins",
        })
    }
}
