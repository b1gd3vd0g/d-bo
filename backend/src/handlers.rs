//! This module is home to the **handler layer** of the application. The handler layer is
//! responsible for parsing information from incoming HTTP requests, calling the appropriate
//! function from the service layer, and mapping the result to an appropriate HTTP response.

pub mod player_handlers;
pub mod request_bodies;
pub mod responses;
