//! This module holds **adapters**, which abstract away interaction with external crates. This
//! causes function calls to be more concise, avoids redundant code, and allows only for **safe,
//! permitted** interaction with the external crates. Adapters map resulting errors to a
//! `DBoResult`, leading to consistency and brevity within the codebase.

pub mod email;
pub mod hashing;
pub mod mongo;
pub mod repositories;
