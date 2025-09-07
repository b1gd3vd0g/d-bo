//! This module is home to the **service layer** of the application. The service layer handles
//! orchestration between repositories, as well as other side-effects of an HTTP request.
//!
//! Service functions are passed in everything they need - repositories, information provided in the
//! HTTP request - and map it to an appropriate DBoError, which can be mapped to an HTTP response
//! by the handlers.

pub mod player_service;
