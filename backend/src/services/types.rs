pub struct LoginTokenInfo {
    access_token: String,
    refresh_token_id: String,
    refresh_token_secret: String,
}

impl LoginTokenInfo {
    pub fn new(access_token: &str, refresh_token_id: &str, refresh_token_secret: &str) -> Self {
        Self {
            access_token: String::from(access_token),
            refresh_token_id: String::from(refresh_token_id),
            refresh_token_secret: String::from(refresh_token_secret),
        }
    }
}
