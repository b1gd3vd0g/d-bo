use serde::Serialize;

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

impl MessageResponse {
    pub fn new(message: &str) -> Self {
        Self {
            message: String::from(message),
        }
    }
}

#[derive(Serialize)]
pub struct ExistingFieldViolationResponse {
    username: bool,
    email: bool,
}

impl ExistingFieldViolationResponse {
    pub fn new(username: bool, email: bool) -> Self {
        Self {
            username: username,
            email: email,
        }
    }
}
