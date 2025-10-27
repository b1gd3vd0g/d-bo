//! This module handles all services related to **player accounts**.

use regex::Regex;

use crate::{
    adapters::{
        email::{
            send_change_email_confirmation_email, send_change_email_warning_email,
            send_change_password_email, send_change_username_email, send_lockout_email,
            send_registration_email, send_request_login_assistance_email,
        },
        hashing::{generate_secret, verify_secret},
        jwt::generate_access_token,
        repositories::{Repository, counter_id::CounterId},
    },
    errors::{AuthnFailureReason, DBoError, DBoResult},
    handlers::{request_bodies::AccountIdentifier, responses::SafePlayerResponse},
    models::{
        Collectible, ConfirmationToken, Counter, Identifiable, Player, RefreshToken, ResetToken,
        UndoToken,
        submodels::{Gender, LanguagePreference, UndoTokenType},
    },
    services::types::LoginTokenInfo,
};

pub struct PlayerService {}

impl PlayerService {
    /// Create a new player account in the database.
    ///
    /// 1. Validate the input, ensuring uniqueness constraints are met.
    /// 2. Add the player document to the database.
    /// 3. Create a new confirmation token for the player and add to the database.
    /// 4. Send a confirmation email to the provided email address.
    /// 5. Increment the `AccountsRegistered` counter.
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
    /// - `TimeZoneParseError` if the `time_zone` cannot be parsed
    /// - `InvalidEmailAddress` if the user's email address could not be parsed into a Mailbox
    ///   (after already passing validation checks - this is not likely)
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
        time_zone: &str,
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
            time_zone,
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

    /// Confirm a newly registered player account.
    ///
    /// 1. Find the player by their id, and ensure that the account is not already confirmed.
    /// 2. Find the token by its id, and ensure that it matches the player and is unexpired.
    /// 3. Delete the token.
    /// 4. Confirm the player's account.
    /// 5. Increment the `AccountsConfirmed` counter.
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
    /// - `PersistentTokenExpired` if the confirmation token is expired (older than 15 minutes)
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
                return Err(DBoError::missing_document(Player::collection_name()));
            }
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::missing_document(
                    ConfirmationToken::collection_name(),
                ));
            }
        };

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        if token.expired() {
            return Err(DBoError::PersistentTokenExpired);
        }

        tokens.delete(token.id()).await?;
        players.confirm(player.id()).await?;
        counters
            .increment_counter(CounterId::AccountsConfirmed)
            .await?;

        Ok(())
    }

    /// Reject the creation of a player account.
    ///
    /// 1. Find a player by their id, and ensure that the account is not already confirmed.
    /// 2. Find the token by its id, and ensure that it matches the player.
    ///     - It does not matter if the token is expired for this request.
    /// 3. Delete the player account.
    /// 4. Delete the token.
    /// 5. Increment the `AccountsRejected` counter.
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
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::missing_document(
                    ConfirmationToken::collection_name(),
                ));
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

    /// Verify a player's login credentials, and provide them with fresh authentication tokens.
    ///
    /// 1. Find the player by their username *or* email address.
    /// 2. Ensure that the account is **confirmed** and **unlocked**.
    /// 3. Verify the password against the stored hash. If that fails:
    ///     1. Increment the **application**'s `FailedLogins` counter.
    ///     2. Increment the **player**'s `failed_login` counter.
    ///     3. Lock the player's account if the player's failed login count exceeds 4.
    ///     4. If the account becomes locked, send a lockout notification email to the player.
    ///     5. Return the appropriate error.
    /// 4. Generate a new Access Token to authenticate the player.
    /// 5. Generate a persistent Refresh Token and store it in the database.
    /// 6. Record the successful login in the player document.
    /// 7. Increment the application's `Logins` counter.
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
    /// - `AuthenticationFailure(BadLoginCredentials)` if the username/email and password do not
    ///   match our records
    /// - `InternalConflict` if the account is unconfirmed.
    /// - `AccountLocked` if either the account is already locked, or if authentication failed for a
    ///   fifth (or greater) time, resulting in a new lockout.
    /// - `MissingDocument` in the *extremely* unlikely case that the player document gets deleted
    ///   midway through this request and cannot be found when trying to update it.
    /// - `InvalidEmailAddress` if the lockout email cannot be sent because the player's stored
    ///   email address cannot be parsed into a mailbox.
    /// - `TimeZoneParseError` in the case that a player's stored time zone cannot be parsed. This
    ///   would indicate data corruption! Also, if this happens, it means that this request has
    ///   resulted in a lockout (although that information will not be passed along to the client)
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
                return Err(DBoError::AuthenticationFailure(
                    AuthnFailureReason::BadLoginCredentials,
                ));
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
                    &time.to_chrono(),
                    player.time_zone(),
                    player.preferred_language(),
                )
                .await?;
                return Err(DBoError::AccountLocked(time.to_chrono()));
            } else {
                return Err(DBoError::AuthenticationFailure(
                    AuthnFailureReason::BadLoginCredentials,
                ));
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

    /// Resend a new registration email to the player.
    ///
    /// 1. Find the player by their id, and ensure that the account is not already confirmed.
    /// 2. Find the token by its id, and ensure that it matches with the player.
    /// 3. Replace the old confirmation token with a newly generated token.
    /// 4. Resend the registration email to the player, containing the new token's credentials.
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
    /// - `InvalidEmailAddress` if the email cannot be sent because a player's email address cannot
    ///   be parsed into a Mailbox
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
                return Err(DBoError::missing_document(Player::collection_name()));
            }
        };

        if player.confirmed() {
            return Err(DBoError::InternalConflict);
        }

        let old_token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::missing_document(
                    ConfirmationToken::collection_name(),
                ));
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

    /// Refresh a players authentication tokens.
    ///
    /// 1. Parse the cookie value to find the ID and secret
    /// 2. Find the refresh token by its ID.
    /// 3. Confirm that the token is unexpired.
    /// 4. Verify that the secret matches the stored hash.
    /// 5. Find the associated player account.
    /// 6. Generate a new Access Token.
    /// 7. Replace the old token in the database with a newly generated refresh token.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The RefreshToken repository
    /// - `cookie_value`: The value of the refresh_token cookie (should be like `"{id}:{secret}"`)
    ///
    /// ### Returns
    /// The information related to both of the created access tokens.
    ///
    /// ### Errors
    /// - `AuthenticationFailure(_)`:
    ///   - `NonParseableCookie` if the cookie value cannot be parsed.
    ///   - `BadCookieCredentials` if the cookie's id and/or secret do not match the database.
    ///   - `ExpiredRefreshToken` if the refresh token is expired.
    ///   - `PlayerNotFound` if the associated player account cannot be found.
    /// - `MissingDocument` if the *old* refresh token cannot be found midway through the request
    ///   when attempting to replace it.
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
            None => {
                return Err(DBoError::AuthenticationFailure(
                    AuthnFailureReason::NonParseableCookie,
                ));
            }
        };

        let token = match tokens.find_by_id(&token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::AuthenticationFailure(
                    AuthnFailureReason::BadCookieCredentials,
                ));
            }
        };

        if token.expired() {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::ExpiredRefreshToken,
            ));
        }

        if !verify_secret(&secret, token.secret())? {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::BadCookieCredentials,
            ));
        }

        let player = match players.find_by_id(token.player_id()).await? {
            Some(p) => p,
            None => {
                return Err(DBoError::AuthenticationFailure(
                    AuthnFailureReason::PlayerNotFound,
                ));
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

    /// Delete a player's own account.
    ///
    /// 1. Identify a player by their access JWT.
    /// 2. Verify that their password matches the stored hash associated with their account.
    /// 3. Delete the document.
    /// 4. Increment the `AccountsDeleted` counter.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository
    /// - `counters`: The Counter Repository
    /// - `jwt`: The player's access JWT
    /// - `password`: The player's password
    ///
    /// ### Errors
    /// - `AuthenticationFailure(_)`:
    ///   - `BadAuthenticationToken` if the JWT cannot be parsed.
    ///   - `ExpiredAuthenticationToken` if the JWT is expired.
    ///   - `PlayerNotFound` if the player document associated with the access token was missing.
    ///   - `PrematureAuthenticationToken` if the JWT was created before a player's sessions were
    ///     invalidated.
    ///   - `BadPassword` if the password does not match the identified player document.
    /// - `AdapterError` if a database query fails, or if the token cannot be decoded due to a
    ///   server-side error.
    pub async fn delete_player_account(
        players: &Repository<Player>,
        counters: &Repository<Counter>,
        jwt: &str,
        password: &str,
    ) -> DBoResult<()> {
        let player = players.find_by_token(jwt).await?;

        if !verify_secret(password, player.password())? {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::BadPassword,
            ));
        }

        players.delete(player.id()).await?;
        counters
            .increment_counter(CounterId::AccountsDeleted)
            .await?;

        Ok(())
    }

    /// Change a player's username.
    ///
    /// 1. Find the player by their access JWT.
    /// 2. Verify that their password matches the stored hash in the player's account.
    /// 3. Update the player's username:
    ///     - Ensure it is valid and unique.
    ///     - Update the document, invalidating their existing sessions.
    /// 4. Send a notification email to the player, informing them that their username has changed.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The Refresh Token repository
    /// - `jwt`: The player's access token
    /// - `password`: The player's password
    /// - `new_username`: The player's new username.
    ///
    /// ### Errors
    /// - `AuthenticationFailure(_)`:
    ///   - `BadAuthenticationToken` if the JWT cannot be parsed.
    ///   - `ExpiredAuthenticationToken` if the JWT is expired.
    ///   - `PlayerNotFound` if the player document associated with the access token was missing.
    ///   - `PrematureAuthenticationToken` if the JWT was created before a player's sessions were
    ///     invalidated.
    ///   - `BadPassword` if the password does not match the identified player document.
    /// - `InvalidPlayerInfo` if the new username is not valid.
    /// - `UniquenessViolation` if the new username is not case-insensitively unique.
    /// - `MissingDocument` if midway through the request, the player cannot be found.
    /// - `InvalidEmailAddress` if the email cannot be sent because a player's stored email address
    ///   cannot be parsed into a Mailbox
    /// - `AdapterError` if a database query fails, or if the token cannot be decoded due to a
    ///   server-side error, or if the player's stored hash could not be parsed, or if the
    ///   notification email cannot be sent due to a server-side error.
    pub async fn change_username(
        players: &Repository<Player>,
        tokens: &Repository<RefreshToken>,
        jwt: &str,
        password: &str,
        new_username: &str,
    ) -> DBoResult<()> {
        let player = players.find_by_token(jwt).await?;

        if !verify_secret(password, player.password())? {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::BadPassword,
            ));
        }

        players.update_username(player.id(), new_username).await?;
        tokens.delete_player_tokens(player.id()).await?;

        send_change_username_email(
            player.email(),
            player.username(),
            new_username,
            player.preferred_language(),
            player.gender(),
        )
        .await?;

        Ok(())
    }

    /// Change a player's **proposed** email address.
    ///
    /// 1. Find the player by their access JWT.
    /// 2. Confirm that their password matches the hash stored in the database.
    /// 3. Update the player's **proposed** email address:
    ///     - Ensure that it is valid and unique.
    ///     - Update the document.
    /// 4. Create a new UndoToken for the player and store it in the database.
    /// 5. Create a new ConfirmationToken for the player and store it in the database.
    /// 6. Send a *warning* email to the player's **current** email address, informing them of the
    ///    proposed change and providing a link to undo the pending change.
    /// 7. Send a *confirmation* email to the player's **proposed** email address, allowing them to
    ///    confirm the new email address and officially replace the current email.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `conf_tokens`: The Confirmation Token repository
    /// - `undo_tokens`: The Undo Token repository
    /// - `jwt`: The player's access token
    /// - `password`: The player's password
    /// - `new_email`: The player's new proposed email address
    ///
    /// ### Errors
    /// - `AuthenticationFailure(_)`:
    ///   - `BadAuthenticationToken` if the JWT cannot be parsed.
    ///   - `ExpiredAuthenticationToken` if the JWT is expired.
    ///   - `PlayerNotFound` if the player document associated with the access token was missing.
    ///   - `PrematureAuthenticationToken` if the JWT was created before a player's sessions were
    ///     invalidated.
    ///   - `BadPassword` if the password does not match the identified player document.
    /// - `InvalidPlayerInfo` if the new email is not valid.
    /// - `UniquenessViolation` if the new email is not case-insensitively unique.
    /// - `InvalidEmailAddress` if either the *new* email address **or** the currently stored email
    ///   address cannot be parsed into a Mailbox.
    /// - `AdapterError` if a database query fails, or if the token cannot be decoded due to a
    ///   server-side error, or if the player's stored hash could not be parsed, or if the
    ///   notification email cannot be sent due to a server-side error.
    pub async fn change_proposed_email(
        players: &Repository<Player>,
        conf_tokens: &Repository<ConfirmationToken>,
        undo_tokens: &Repository<UndoToken>,
        jwt: &str,
        password: &str,
        new_email: &str,
    ) -> DBoResult<()> {
        let player = players.find_by_token(jwt).await?;

        if !verify_secret(password, player.password())? {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::BadPassword,
            ));
        }

        players
            .update_proposed_email(player.id(), new_email)
            .await?;

        let undo_token = UndoToken::new(player.id(), &UndoTokenType::Email);
        undo_tokens.insert(&undo_token).await?;

        let conf_token = ConfirmationToken::new(player.id());
        conf_tokens.insert(&conf_token).await?;

        send_change_email_warning_email(
            player.username(),
            player.email(),
            new_email,
            player.id(),
            undo_token.id(),
            player.preferred_language(),
        )
        .await?;

        send_change_email_confirmation_email(
            player.username(),
            player.email(),
            new_email,
            player.id(),
            conf_token.id(),
            undo_token.id(),
            player.preferred_language(),
            player.pronoun(),
        )
        .await?;

        Ok(())
    }

    /// Confirm a player's proposed email address.
    ///
    /// 1. Find the player by their id.
    /// 2. Find the confirmation token by its id.
    /// 3. Confirm that the token is unexpired, and that it matches with the same player.
    /// 4. Confirm the player's **proposed** email address, making it the **email**.
    ///     - Ensure that the `proposed_email` field exists, and that it is valid and unique.
    ///     - Invalidate the player's current sessions.
    /// 5. Delete the used confirmation token from the database.
    /// 6. Delete the UndoToken that was created when the player's new email address was proposed.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `conf_tokens`: The Confirmation Token repository
    /// - `undo_tokens`: The Undo Token repository
    /// - `player_id`: The player's unique identifier
    /// - `token_id`: The confirmation token's unique identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if the player or the confirmation token cannot be found
    /// - `PersistentTokenExpired` if the confirmation token is expired
    /// - `RelationalConflict` if the token does not match the player
    /// - `InternalConflict` if the player does not have a proposed email address
    /// - `InvalidPlayerInfo` if the proposed email address cannot be validated
    /// - `UniquenessViolation` if the proposed email address is not unique
    /// - `AdapterError` if a database query fails
    pub async fn confirm_proposed_email(
        players: &Repository<Player>,
        conf_tokens: &Repository<ConfirmationToken>,
        undo_tokens: &Repository<UndoToken>,
        player_id: &str,
        token_id: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let token = match conf_tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => {
                return Err(DBoError::missing_document(
                    ConfirmationToken::collection_name(),
                ));
            }
        };

        if token.expired() {
            return Err(DBoError::PersistentTokenExpired);
        }

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        players.confirm_proposed_email(player.id()).await?;
        conf_tokens.delete(token.id()).await?;
        undo_tokens
            .delete_by_player_and_func(player.id(), &UndoTokenType::Email)
            .await?;

        Ok(())
    }

    /// Change a player's password.
    ///
    /// 1. Find the player by their access JWT.
    /// 2. Verify that their password matches the stored hash in the database.
    /// 3. Update the player's password:
    ///     - Ensure that it is valid
    ///     - Ensure that it does not match any of their last five passwords.
    ///     - Rotate the player's `last_passwords`, and update their current `password`.
    ///     - Invalidate the player's existing sessions.
    /// 4. Create a new UndoToken for the player, and add it to the database.
    /// 5. Send the player a notification email, providing them with a link to reset their password
    ///    without logging in, which is good for 24 hours.
    ///
    /// ### Arguments
    /// - `players`: The Player repository
    /// - `tokens`: The Undo Token repository
    /// - `jwt`: The player's access token
    /// - `old_password`: The player's current password
    /// - `new_password`: The player's new password to be set
    ///
    /// ### Errors
    /// - `AuthenticationFailure(_)`:
    ///   - `BadAuthenticationToken` if the JWT cannot be parsed.
    ///   - `ExpiredAuthenticationToken` if the JWT is expired.
    ///   - `PlayerNotFound` if the player document associated with the access token was missing.
    ///   - `PrematureAuthenticationToken` if the JWT was created before a player's sessions were
    ///     invalidated.
    ///   - `BadPassword` if the password does not match the identified player document.
    /// - `InvalidPlayerInfo` if the password is not valid
    /// - `MissingDocument` if midway through the function, the player account can no longer be
    ///   found.
    /// - `InternalConflict` if the new password matches any of the player's last five passwords
    /// - `InvalidEmailAddress` if the player's email address cannot be parsed into a Mailbox
    /// - `AdapterError` if a database query fails, or if the access token cannot be decoded due to
    ///   a server-side error, or if any of the player's stored hashes cannot be decoded, or if the
    ///   new password cannot be hashed, or if the email cannot be sent due to a server-side error.
    pub async fn change_password(
        players: &Repository<Player>,
        tokens: &Repository<UndoToken>,
        jwt: &str,
        old_password: &str,
        new_password: &str,
    ) -> DBoResult<()> {
        let player = players.find_by_token(jwt).await?;

        if !verify_secret(old_password, player.password())? {
            return Err(DBoError::AuthenticationFailure(
                AuthnFailureReason::BadPassword,
            ));
        }

        players.update_password(player.id(), new_password).await?;

        let token = UndoToken::new(player.id(), &UndoTokenType::Password);
        tokens.insert(&token).await?;

        send_change_password_email(
            player.email(),
            player.username(),
            player.id(),
            token.id(),
            player.preferred_language(),
            player.pronoun(),
        )
        .await?;

        Ok(())
    }

    /// Use an undo token to reject a player's proposed email change.
    ///
    /// 1. Find the player by their ID.
    /// 2. Find the token by its ID.
    /// 3. Ensure that the token is unexpired and that it matches with the same player account.
    /// 4. Remove the player's `proposed_email` field, if it exists.
    /// 5. Delete the used undo token from the database.
    /// 6. Delete the confirmation token that was created when the new email address was proposed.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository
    /// - `undo_tokens`: The UndoToken Repository
    /// - `conf_tokens`: The ConfirmationToken Repository
    /// - `player_id`: The player's identifier
    /// - `token_id`: The undo token's identifier
    ///
    /// ### Errors
    /// - `MissingDocument` if the player or undo token cannot be found.
    /// - `PersistentTokenExpired` if the undo token is expired.
    /// - `RelationalConflict` if the undo token does not match the same player.
    /// - `InternalConflict` if the player does not have a `proposed_email` field.
    /// - `AdapterError` if any database query should fail.
    pub async fn reject_proposed_email(
        players: &Repository<Player>,
        undo_tokens: &Repository<UndoToken>,
        conf_tokens: &Repository<ConfirmationToken>,
        player_id: &str,
        token_id: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let token = match undo_tokens.find_by_id(token_id).await? {
            Some(tok) => tok,
            None => return Err(DBoError::missing_document(UndoToken::collection_name())),
        };

        if token.expired() {
            return Err(DBoError::PersistentTokenExpired);
        }

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        players.reject_proposed_email(player.id()).await?;
        undo_tokens.delete(token.id()).await?;
        conf_tokens.delete_by_player_id(player.id()).await?;

        Ok(())
    }

    /// Use an undo token to reset a player's password following an unauthorized password change.
    ///
    /// 1. Find the player by their ID.
    /// 2. Find the undo token by its ID.
    /// 3. Ensure that the token is unexpired and that it represents the same player account.
    /// 4. Change the player's password to the new one:
    ///     - Ensure that it is valid
    ///     - Ensure that it does not match any of their last five passwords.
    ///     - Rotate the player's `last_passwords`, and update their current `password`.
    ///     - Invalidate the player's existing sessions.
    /// 5. Delete the used undo token from the database.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository
    /// - `tokens`: The UndoToken Repository
    /// - `player_id`: The player's identifier
    /// - `token_id`: The undo token's identifier
    /// - `new_password`: The player's newly proposed password.
    ///
    /// ### Errors
    /// - `MissingDocument` if either the player or the undo token cannot be found.
    /// - `PersistentTokenExpired` if the undo token has expired.
    /// - `RelationalConflict` if the undo token does not represent the same player account.
    /// - `InvalidPlayerInfo` if the proposed password is not valid.
    /// - `InternalConflict` if the new password matches any of the player's last five passwords.
    /// - `AdapterError` if any database query should fail, or if any previous passwords cannot be
    ///   parsed, or if the new password cannot be hashed.
    pub async fn reset_password_following_rejecting_change(
        players: &Repository<Player>,
        tokens: &Repository<UndoToken>,
        player_id: &str,
        token_id: &str,
        new_password: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => return Err(DBoError::missing_document(UndoToken::collection_name())),
        };

        if token.expired() {
            return Err(DBoError::PersistentTokenExpired);
        }

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        players.update_password(player.id(), new_password).await?;
        tokens.delete(token.id()).await?;

        Ok(())
    }

    /// Send an email providing a player with their username and a link to reset their password.
    ///
    /// 1. Identify the account by the username or email address provided in `id`.
    /// 2. Create a new Reset Token, and store it in the database.
    /// 3. Send an email to the player, providing them with their username and a link to reset their
    ///    password, valid for 15 minutes.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository.
    /// - `tokens`: The ResetToken Repository.
    /// - `id`: The player's username or email address.
    ///
    /// ### Errors
    /// - `MissingDocument` if the player cannot be found by their identifier.
    /// - `InvalidEmailAddress` if the player's stored email address cannot be parsed into a
    ///   Mailbox.
    /// - `AdapterError` if a database query should fail, or if the email cannot be sent due to a
    ///   server-side error.
    pub async fn request_login_assistance(
        players: &Repository<Player>,
        tokens: &Repository<ResetToken>,
        id: &AccountIdentifier,
    ) -> DBoResult<()> {
        let option = match id {
            AccountIdentifier::Username(val) => players.find_by_username(&val).await?,
            AccountIdentifier::Email(val) => players.find_by_email(&val).await?,
        };

        let player = match option {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let token = ResetToken::new(player.id());
        tokens.insert(&token).await?;

        send_request_login_assistance_email(
            player.email(),
            player.username(),
            player.id(),
            token.id(),
            player.preferred_language(),
            player.pronoun(),
        )
        .await?;

        Ok(())
    }

    /// Use a reset token to reset a player's password when they have forgotten it.
    ///
    /// 1. Find the player by their ID.
    /// 2. Find the reset token by its ID.
    /// 3. Ensure the reset token is unexpired and that it represents the same player account.
    /// 4. Update the player's password:    
    ///     - Ensure that it is valid
    ///     - Ensure that it does not match any of their last five passwords.
    ///     - Rotate the player's `last_passwords`, and update their current `password`.
    ///     - Invalidate the player's existing sessions.
    /// 5. Delete the used reset token from the database.
    ///
    /// ### Arguments
    /// - `players`: The Player Repository
    /// - `tokens`: The ResetToken Repository
    /// - `player_id`: The player's unique identifier.
    /// - `token_id`: The ResetToken's unique identifier.
    /// - `new_password`: The player's proposed new password.
    ///
    /// ### Errors
    /// - `MissingDocument` if the player or reset token cannot be found.
    /// - `PersistentTokenExpired` if the reset token is expired.
    /// - `RelationalConflict` if the reset token does not represent the same player account.
    /// - `InvalidPlayerInfo` if the password is not valid.
    /// - `InternalConflict` if the password matches any of the player's last five used passwords.
    /// - `AdapterError` if a database query fails, or if one of the player's stored hashes cannot
    ///   be parsed, or if the new password cannot be hashed.
    pub async fn reset_forgotten_password(
        players: &Repository<Player>,
        tokens: &Repository<ResetToken>,
        player_id: &str,
        token_id: &str,
        new_password: &str,
    ) -> DBoResult<()> {
        let player = match players.find_by_id(player_id).await? {
            Some(p) => p,
            None => return Err(DBoError::missing_document(Player::collection_name())),
        };

        let token = match tokens.find_by_id(token_id).await? {
            Some(t) => t,
            None => return Err(DBoError::missing_document(ResetToken::collection_name())),
        };

        if token.expired() {
            return Err(DBoError::PersistentTokenExpired);
        }

        if token.player_id() != player.id() {
            return Err(DBoError::RelationalConflict);
        }

        players.update_password(player.id(), new_password).await?;
        tokens.delete(token.id()).await?;

        Ok(())
    }
}
