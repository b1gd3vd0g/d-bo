//! This module handles all services related to **player accounts**.

use crate::{
    adapters::{
        email::send_registration_email,
        hashing::{generate_secret, verify_secret},
        jwt::generate_access_token,
        repositories::{Repository, counter_id::CounterId},
    },
    errors::{DBoError, DBoResult},
    handlers::responses::SafePlayerResponse,
    models::{
        ConfirmationToken, Counter, Identifiable, Player, RefreshToken,
        submodels::{Gender, LanguagePreference},
    },
    services::types::LoginTokenInfo,
};

pub struct PlayerService {}

impl PlayerService {
    /// Create a new player account in the database, create a new confirmation token for them to
    /// use, and send a confirmation email to the provided email address.
    ///
    /// ### Arguments
    /// - `players`: The player repository
    /// - `tokens`: The confirmation tokens repository
    /// - `username`: The requested username
    /// - `password`: The requested password
    /// - `email`: The requested email address
    /// - `gender`: The player's gender
    /// - `preferred_language`: The player's preferred language
    /// - `pronoun`: The player's preferred pronouns. This is only used in the case of Spanish
    ///   speaking non-binary players; all other players' pronouns will match with their gender
    ///   automatically.
    ///
    /// ### Returns
    /// The created player's safe information.
    ///
    /// ### Errors
    /// - `InvalidPlayerInfo` if the username, password, or email cannot pass validation.
    /// - `UniquenessViolation` if the username or email are not case-insensitively unique.
    /// - `ServerSideError` if the email templates cannot be found.
    /// - `AdapterError` if a database query fails, if the password cannot be hashed, or if the
    ///   confirmation email could not be sent
    pub async fn register_player(
        players: &Repository<Player>,
        tokens: &Repository<ConfirmationToken>,
        counters: &Repository<Counter>,
        username: &str,
        password: &str,
        email: &str,
        gender: &Gender,
        preferred_language: &LanguagePreference,
        pronoun: &Option<Gender>,
    ) -> DBoResult<SafePlayerResponse> {
        let assumed_pronoun = match (gender, preferred_language) {
            (Gender::Other, LanguagePreference::Spanish) => match pronoun {
                Some(p) => p,
                None => gender,
            },
            _ => gender,
        };

        let player = Player::new(
            username,
            password,
            email,
            gender,
            preferred_language,
            assumed_pronoun,
        )?;
        players.insert(&player).await?;

        let token = ConfirmationToken::new(player.id());
        tokens.insert(&token).await?;

        send_registration_email(
            email,
            username,
            token.id(),
            player.id(),
            preferred_language,
            assumed_pronoun,
        )
        .await?;

        counters
            .increment_counter(CounterId::AccountsRegistered)
            .await?;

        Ok(SafePlayerResponse::from(&player))
    }

    /// Attempt to verify a player's login information. Create an access token and a refresh token
    /// for secure authentication. Store the refresh token in the database.
    ///
    /// ### Arguments
    /// - `players`: The player repository
    /// - `tokens`: The refresh token repository
    /// - `username_or_email`: The player's username or email address
    /// - `password`: The player's password
    ///
    /// ### Returns
    /// The information related to the created authentication tokens
    ///
    /// ### Errors
    /// - `AuthenticationFailure` if the username/email and password do not match our records
    /// - `AdapterError` if a database query fails, if the password or refresh token
    ///   secret cannot be hashed, or if the access token cannot be created.
    pub async fn login(
        players: &Repository<Player>,
        tokens: &Repository<RefreshToken>,
        username_or_email: &str,
        password: &str,
    ) -> DBoResult<LoginTokenInfo> {
        let option = players.find_by_username_or_email(username_or_email).await?;

        let player = if let Some(p) = option {
            p
        } else {
            return Err(DBoError::AuthenticationFailure);
        };

        if !verify_secret(password, player.password())? {
            return Err(DBoError::AuthenticationFailure);
        }

        let access_token = generate_access_token(player.id())?;
        let refresh_secret = generate_secret();

        let refresh_token = RefreshToken::new(player.id(), &refresh_secret)?;
        tokens.insert(&refresh_token).await?;

        Ok(LoginTokenInfo::new(
            &access_token,
            refresh_token.id(),
            &refresh_secret,
        ))
    }
}
