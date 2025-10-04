//! This module is an adapter over the `lettre` crate, allowing for the sending of various types of
//! emails necessary within the application.

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use regex::Regex;

use crate::{
    config::{assets::ASSETS, environment::ENV},
    errors::DBoResult,
    models::submodels::{Gender, LanguagePreference},
};

/// Replace all the value placeholders (`{{<value>}}`) within the confirmation email template with
/// actual values, to send a meaningful and complete email to the player.
///
/// ### Arguments
/// - `template`: The template to fill in (could be plaintext or HTML)
/// - `username`: The receiving player's username
/// - `token_id`: The identifier of the confirmation token which can confirm the player's account
///
/// ### Returns
/// A string with all placeholders replaced with the necessary information
#[doc(hidden)]
fn fill_confirmation_template(template: &str, username: &str, token_id: &str) -> String {
    template
        .replace("{{USERNAME}}", username)
        .replace("{{TOKEN_ID}}", token_id)
        .replace("{{FRONTEND_URL}}", &ENV.frontend_url)
        .replace("{{BIGDEVDOG_LOGO}}", &ASSETS.images.bigdevdog_logo.cid())
        .replace("{{D_BO_LOGO}}", &ASSETS.images.d_bo_logo.cid())
}

/// Replace all the gendered placeholders (`**<m>/<f>/<nb>**`) within an email template with
/// the actual values corresponding to a player's gender, to send a meaningful and grammatically
/// correct email to the players. This is **most often** used for Spanish correspondences, but is
/// versatile.
///
/// ### Arguments
/// - `template`: The template to fill in
/// - `gender`: The Gender associated with the player's `pronoun`.
///
/// ### Returns
/// A string with all placeholders replaced with the correct gendered language.
fn fill_gendered_template(template: &str, gender: &Gender) -> String {
    let regex = Regex::new(r"\*\*([^/]+)/([^/]+)/([^/]+)\*\*").unwrap();

    regex
        .replace_all(template, |captures: &regex::Captures| match gender {
            Gender::Male => captures[1].to_string(),
            Gender::Female => captures[2].to_string(),
            Gender::Other => captures[3].to_string(),
        })
        .to_string()
}

/// Send a registration email to the player, providing them with a link to confirm their email
/// address and activate their account, so they may start to utilize the functionality of the
/// application.
///
/// ### Arguments
/// - `player_email`: The email address to send to
/// - `username`: The player's username
/// - `token_id`: The confirmation token id
/// - `language`: The language to send the email in
/// - `pronoun`: Specifies gender-specific language in the Spanish version of the email
///
/// ### Errors
/// - `ServerSideError`: if the email template files cannot be found.
/// - `AdapterError`: if the email cannot be constructed or sent.
pub async fn send_registration_email(
    player_email: &str,
    username: &str,
    token_id: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let variants = match language {
        LanguagePreference::English => &ASSETS.templates.registration.en,
        LanguagePreference::Spanish => &ASSETS.templates.registration.es,
    };

    let mut html_message = fill_confirmation_template(&variants.html, username, token_id);
    let mut txt_message = fill_confirmation_template(&variants.txt, username, token_id);

    let subject = match language {
        LanguagePreference::Spanish => {
            html_message = fill_gendered_template(&html_message, pronoun);
            txt_message = fill_gendered_template(&txt_message, pronoun);
            "¡Confirme su dirección de correo electrónico para empezar a jugar D-Bo!"
        }
        LanguagePreference::English => "Confirm your email address to start playing D-Bo!",
    };

    let message = Message::builder()
        .from("d-bo@bigdevdog.com".parse().unwrap())
        .to(player_email.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(txt_message),
                )
                .multipart(
                    MultiPart::related()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_message),
                        )
                        .singlepart(Attachment::new_inline(ASSETS.images.d_bo_logo.cid()).body(
                            ASSETS.images.d_bo_logo.bytes(),
                            ASSETS.images.d_bo_logo.mime_type(),
                        ))
                        .singlepart(
                            Attachment::new_inline(ASSETS.images.bigdevdog_logo.cid()).body(
                                ASSETS.images.bigdevdog_logo.bytes(),
                                ASSETS.images.bigdevdog_logo.mime_type(),
                            ),
                        ),
                ),
        )?;

    let smtp_credentials = Credentials::new(ENV.smtp_username.clone(), ENV.smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&ENV.smtp_host)
        .unwrap()
        .credentials(smtp_credentials)
        .build();

    mailer.send(message).await?;

    Ok(())
}
