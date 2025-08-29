use crate::mongo::models::Player;

impl Player {
    pub fn new(
        player_id: &str,
        username: &str,
        password: &str,
        email: &str,
        confirmed: bool,
    ) -> Self {
        Self {
            player_id: String::from(player_id),
            username: String::from(username),
            password: String::from(password),
            email: String::from(email),
            confirmed: confirmed,
        }
    }

    pub fn player_id(&self) -> String {
        self.player_id.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }

    pub fn email(&self) -> String {
        self.email.clone()
    }

    pub fn confirmed(&self) -> bool {
        self.confirmed
    }
}
