//! This module handles configuration of Lazy variables defining constants used throughout the
//! application. These Lazy variables are forced on app startup.
//!
//! This setup prevents the app from panicking or throwing server-side errors after the HTTP router
//! has begun to listen.

pub mod assets;
pub mod environment;
