use std::env;

use dotenvy::dotenv;
use once_cell::sync::Lazy;

/// Holds all of the environment variables used within the code.
pub struct Environment {
    pub authn_token_secret: String,
    pub frontend_url: String,
    pub mongo_username: String,
    pub mongo_password: String,
    pub mongo_server: String,
    pub mongo_dbname: String,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
}

/// Find an environment variable which **must** be defined externally.
///
/// ### Arguments
/// - `varname`: The name of the environment variable.
///
/// ### Panics
/// If the environment variable is undefined.
fn secret_var(varname: &str) -> String {
    env::var(varname).expect(&format!(
        r#"Environment variable "{}" is not set!"#,
        varname
    ))
}

/// Try to find an environment variable, but if it cannot be found, set it to a default value.
///
/// **Note**: The default value is **only acceptable in development environments**; in production,
/// **all** environment variables must be defined.
///
/// ### Arguments
/// - `varname`: The name of the environment variable.
/// - `default`: The default value to use in development environments.
///
/// ### Panics
/// If the environment variable is undefined **in a production environment**.
fn default_var(varname: &str, default: &str) -> String {
    env::var(varname).unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            String::from(default)
        } else {
            panic!(r#"Environment variable "{}" must be set in prod!"#, varname);
        }
    })
}

impl Environment {
    /// Configure the environment variable. If the app is running in a development environment, load
    /// the environment variables from a `.env` file first.
    ///
    /// ### Panics
    ///
    /// If **any** of the secret environment variables are undefined, or if a default value is used
    /// **in a production environment**.
    pub fn configure() -> Self {
        if cfg!(debug_assertions) {
            dotenv().ok();
        }

        Self {
            authn_token_secret: secret_var("AUTHN_TOKEN_SECRET"),
            frontend_url: default_var("FRONTEND_URL", "http://localhost:5173"),
            mongo_username: secret_var("MONGO_USERNAME"),
            mongo_password: secret_var("MONGO_PASSWORD"),
            mongo_server: secret_var("MONGO_SERVER"),
            mongo_dbname: secret_var("MONGO_DBNAME"),
            smtp_host: secret_var("SMTP_HOST"),
            smtp_username: secret_var("SMTP_USERNAME"),
            smtp_password: secret_var("SMTP_PASSWORD"),
        }
    }
}

/// Holds all of our environment variables for safe use at any point within the application.
pub static ENV: Lazy<Environment> = Lazy::new(Environment::configure);
