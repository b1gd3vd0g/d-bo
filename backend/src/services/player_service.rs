//! This module handles all services related to **player accounts**.

use regex::Regex;

use crate::{
    adapters::{
        email::{send_lockout_email, send_registration_email},
        hashing::{generate_secret, verify_secret},
        jwt::{decode_access_token, generate_access_token},
        repositories::{Repository, counter_id::CounterId},
    },
    errors::{DBoError, DBoResult},
    handlers::responses::SafePlayerResponse,
    models::{
        Collectible, ConfirmationToken, Counter, Identifiable, Player, RefreshToken,
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

    /// Confirm a player's account. Find a player by their id, ensure that the account is not
    /// already confirmed; find the token by its id, ensure that it matches the same player, and
    /// that it is unexpired; delete the token, confirm the player's account, and increment the
    /// counter.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The Confirmation Token repository
    /// - `counters`: The Counters repository
    /// - `player_id`: The player's unique identifier
    /// - `token_id`: The token's unique identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if either the player or the token could not be found
    /// - `InternalConflict` if the player account is already confirmed
    /// - `RelationalConflict` if the token does not match the player
    /// - `TokenExpired` if the confirmation token is expired (older than 15 minutes)
    /// - `AdapterError` if any database query should fail
    pub async fn confirm_player_account(
        players: &Repository<Player>,
        tokens: &Repository<ConfirmationToken>,
        counters: &Repository<Counter>,
        player_id: &str,
        token_id: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    Player::collection_name(),
                )));
            }
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    ConfirmationToken::collection_name(),
                )));
            }
        };

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        if token.expired() {
            return Err(DBoError::TokenExpired);
        }

        tokens.delete(token.id()).await?;
        players.confirm(player.id()).await?;
        counters
            .increment_counter(CounterId::AccountsConfirmed)
            .await?;

        Ok(())
    }

    /// Reject the creation of a player account. Find a player by their id, and ensure that they are
    /// not already confirmed; find the confirmation token by id, and ensure that it matches the
    /// same player. Delete the player account, delete the token, and increment the counter. If the
    /// player cannot be found, this request will succeed, as that was the point all along. Unlike
    /// account confirmation, account rejection will succeed even if the token is *expired* after 15
    /// minutes.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The ConfirmationToken repository
    /// - `counters`: The Counter repository
    /// - `player_id`: The player's unique identifier
    /// - `token_id`: The token's unique identifier
    ///
    /// ### Errors
    /// - `InternalConflict` if the account is already confirmed
    /// - `MissingDocument` if the token cannot be found
    /// - `RelationalConflict` if the player account does not match the token
    /// - `AdapterError` if any database query should fail
    pub async fn reject_player_account(
        players: &Repository<Player>,
        tokens: &Repository<ConfirmationToken>,
        counters: &Repository<Counter>,
        player_id: &str,
        token_id: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Ok(()),
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    ConfirmationToken::collection_name(),
                )));
            }
        };

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        players.delete(player.id()).await?;
        tokens.delete(token.id()).await?;
        counters
            .increment_counter(CounterId::AccountsRejected)
            .await?;

        Ok(())
    }

    /// Attempt to verify a player's login information. Find the player by username/email, and
    /// ensure that the account is not currently locked. Check the password against the hash in the
    /// database - if it does not match, increment the `failed_login` count, locking the player out
    /// if that count exceeds 4. If the account becomes locked out due to this login attempt, send
    /// an email to the player notifying them that their account has been locked out.
    ///
    /// Upon a login success, generate an access token (a JWT good for 15 minutes) to authenticate
    /// the player. Then generate a persistent refresh token in the database, good for 30 days.
    ///
    /// ### Arguments
    /// - `players`: The player repository
    /// - `tokens`: The refresh token repository
    /// - `username_or_email`: The player's username or email address
    /// - `password`: The player's password
    ///
    /// ### Returns
    /// The information related to both of the created authentication tokens
    ///
    /// ### Errors
    /// - `AuthenticationFailure` if the username/email and password do not match our records
    /// - `InternalConflict` if the account is unconfirmed.
    /// - `AccountLocked` if either the account is already locked, or if authentication failed for a
    ///   fifth (or greater) time, resulting in a new lockout.
    /// - `MissingDocument` in the *extremely* unlikely case that the player document gets deleted
    ///   midway through this request and cannot be found when trying to update it.
    /// - `AdapterError` if a database query fails, if the password or refresh token
    ///   secret cannot be hashed, if the access JWT cannot be created, or if the lockout email
    ///   fails to be sent.
    pub async fn login(
        players: &Repository<Player>,
        tokens: &Repository<RefreshToken>,
        counters: &Repository<Counter>,
        username_or_email: &str,
        password: &str,
    ) -> DBoResult<LoginTokenInfo> {
        let player = match players.find_by_username_or_email(username_or_email).await? {
            Some(p) => p,
            None => {
                counters.increment_counter(CounterId::FailedLogins).await?;
                return Err(DBoError::AuthenticationFailure);
            }
        };

        if !player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        if player.locked() {
            return Err(DBoError::AccountLocked(
                player.locked_until().unwrap().to_chrono(),
            ));
        }

        if !verify_secret(password, player.password())? {
            counters.increment_counter(CounterId::FailedLogins).await?;

            let lockout = players.increment_failed_logins(player.id()).await?;

            if let Some(time) = lockout {
                send_lockout_email(
                    player.email(),
                    player.username(),
                    player.failed_logins() + 1,
                    &time.to_chrono().to_rfc3339(),
                    player.preferred_language(),
                )
                .await?;
                return Err(DBoError::AccountLocked(time.to_chrono()));
            } else {
                return Err(DBoError::AuthenticationFailure);
            }
        }

        let access_token = generate_access_token(player.id())?;

        let refresh_secret = generate_secret();
        let refresh_token = RefreshToken::new(player.id(), &refresh_secret)?;

        tokens.insert(&refresh_token).await?;
        players.record_successful_login(player.id()).await?;
        counters.increment_counter(CounterId::Logins).await?;

        Ok(LoginTokenInfo::new(
            &access_token,
            refresh_token.id(),
            &refresh_secret,
        ))
    }

    /// Resend a new registration email to the player. This happens most likely when a player tries
    /// to confirm their new account, but finds their original confirmation token to be expired.
    ///
    /// Search for the player by id, and make sure that it is not already confirmed. Find the old
    /// confirmation token in the database, generate a new one, and replace it. Finally, resend the
    /// email to the player.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The ConfirmationToken repository
    /// - `player_id`: The player's unique identifier
    /// - `token_id`: The old confirmation token's unique identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if either the player or token cannot be found
    /// - `InternalConflict` if the player account is already confirmed
    /// - `RelationalConflict` if the token is not associated with the same player
    /// - `AdapterError` if a database query should fail, or if the email could not be sent
    pub async fn resend_registration_email(
        players: &Repository<Player>,
        tokens: &Repository<ConfirmationToken>,
        player_id: &str,
        token_id: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    Player::collection_name(),
                )));
            }
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let old_token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    ConfirmationToken::collection_name(),
                )));
            }
        };

        if old_token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        let new_token = ConfirmationToken::new(player.id());
        tokens.insert(&new_token).await?;

        send_registration_email(
            player.email(),
            player.username(),
            new_token.id(),
            player.id(),
            player.preferred_language(),
            player.pronoun(),
        )
        .await?;

        Ok(())
    }

    /// Refresh a players authentication tokens. Parse the cookie to find the ID and secret; find
    /// the refresh token in the database matching the id; verify that the secrets match; confirm
    /// that the token is unexpired; find the associated player account; make a new access token;
    /// replace the old refresh token in the database with a new one.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The RefreshToken repository
    /// - `cookie_value`: The value of the refresh_token cookie (should be like `"{id}:{secret}"`)
    ///
    /// ### Errors
    /// - `InvalidToken` if the cookie value cannot be parsed into an id and a secret
    /// - `AuthenticationFailure` if the token cannot be found, or if the secret does not match
    /// - `TokenExpired` if the token is found but is older than 30 days
    /// - `InternalConflict` if the token has been revoked
    /// - `MissingDocument` if the associated player account cannot be found, or if midway through,
    ///   the old token cannot be found in order to replace it
    /// - `AdapterError` if any database query should fail, or if the secret could not be verified,
    ///   or if the new token cannot be created, or if the new secret could not be hashed.
    pub async fn refresh_authn_tokens(
        players: &Repository<Player>,
        tokens: &Repository<RefreshToken>,
        cookie_value: &str,
    ) -> DBoResult<LoginTokenInfo> {
        let regex = Regex::new(r"([^:]+):([^:]+)").unwrap();

        let (token_id, secret) = match regex.captures(cookie_value) {
            Some(caps) => (caps[1].to_string(), caps[2].to_string()),
            None => return Err(DBoError::InvalidToken),
        };

        let token = match tokens.find_by_id(&token_id).await? {
            Some(t) => t,
            None => return Err(DBoError::AuthenticationFailure),
        };

        if token.expired() {
            return Err(DBoError::TokenExpired);
        }

        if token.revoked() {
            return Err(DBoError::InternalConflict);
        }

        if !verify_secret(&secret, token.secret())? {
            return Err(DBoError::AuthenticationFailure);
        }

        let player = match players.find_by_id(token.player_id()).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    Player::collection_name(),
                )));
            }
        };

        let access_token = generate_access_token(player.id())?;
        let new_secret = generate_secret();
        let new_refresh_token = RefreshToken::new(player.id(), &new_secret)?;

        tokens.replace(token.id(), &new_refresh_token).await?;

        Ok(LoginTokenInfo::new(
            &access_token,
            new_refresh_token.id(),
            &new_secret,
        ))
    }

    /// Delete a player's account. This requires that they have a valid access token to identify
    /// them, and they must also provide their password to further verify their identity. Find the
    /// player by the token, delete the document if the password matches, and increment the counter.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository
    /// - `counters`: The Counter Repository
    /// - `token`: The player's access token JWT
    /// - `password`: The player's password
    ///
    /// ### Errors
    /// - `TokenExpired` if the access token is expired.
    /// - `InvalidToken` if the token cannot be decoded because it is bad.
    /// - `MissingDocument` if the player cannot be identified by the token.
    /// - `AdapterError` if a database query fails, or if the token cannot be decoded due to a
    ///   server-side error.
    pub async fn delete_player_account(
        players: &Repository<Player>,
        counters: &Repository<Counter>,
        token: &str,
        password: &str,
    ) -> DBoResult<()> {
        let payload = decode_access_token(token)?;

        let player = match players.find_by_id(payload.sub()).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::MissingDocument(String::from(
                    Player::collection_name(),
                )));
            }
        };

        if !verify_secret(password, player.password())? {
            return Err(DBoError::AuthenticationFailure);
        }

        players.delete(player.id()).await?;
        counters
            .increment_counter(CounterId::AccountsDeleted)
            .await?;

        Ok(())
    }
}
