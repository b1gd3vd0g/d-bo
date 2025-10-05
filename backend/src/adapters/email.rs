//! This module is an adapter over the `lettre` crate, allowing for the sending of various types of
//! emails necessary within the application.

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    config::{
        assets::{ASSETS, EmailFormatVariants},
        environment::ENV,
    },
    errors::DBoResult,
    models::submodels::{Gender, LanguagePreference},
};

/// The mailer used to send all emails from the official D-Bo email address.
static MAILER: Lazy<AsyncSmtpTransport<Tokio1Executor>> = Lazy::new(|| {
    let credentials = Credentials::new(ENV.smtp_username.clone(), ENV.smtp_password.clone());
    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&ENV.smtp_host)
        .unwrap()
        .credentials(credentials)
        .build()
});

/// The "from" address for messages.
static MAILBOX: Lazy<Mailbox> = Lazy::new(|| "d-bo@bigdevdog.com".parse().unwrap());

struct PlaceholderHelper {
    placeholder: &'static str,
    value: String,
}

impl PlaceholderHelper {
    pub fn new(placeholder: &'static str, value: &str) -> Self {
        Self {
            placeholder: placeholder,
            value: String::from(value),
        }
    }
}

fn replace_placeholders(
    templates: &EmailFormatVariants,
    helpers: &Vec<PlaceholderHelper>,
) -> EmailFormatVariants {
    let mut html = templates.html.clone();
    let mut txt = templates.txt.clone();

    for helper in helpers {
        html = html.replace(helper.placeholder, &helper.value);
        txt = txt.replace(helper.placeholder, &helper.value);
    }

    EmailFormatVariants {
        html: html,
        txt: txt,
    }
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

// //////////// //
// REGISTRATION //
// //////////// //

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
    player_id: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let helpers = vec![
        PlaceholderHelper::new("{{USERNAME}}", username),
        PlaceholderHelper::new("{{TOKEN_ID}}", token_id),
        PlaceholderHelper::new("{{PLAYER_ID}}", player_id),
        PlaceholderHelper::new("{{FRONTEND_URL}}", &ENV.frontend_url),
        PlaceholderHelper::new("{{BIGDEVDOG_LOGO}}", &ASSETS.images.bigdevdog_logo.cid()),
        PlaceholderHelper::new("{{D_BO_LOGO}}", &ASSETS.images.d_bo_logo.cid()),
    ];

    let mut variants =
        replace_placeholders(&ASSETS.templates.registration.language(language), &helpers);

    let subject = match language {
        LanguagePreference::Spanish => {
            variants.html = fill_gendered_template(&variants.html, pronoun);
            variants.txt = fill_gendered_template(&variants.txt, pronoun);
            "¡Confirme su dirección de correo electrónico para empezar a jugar D-Bo!"
        }
        LanguagePreference::English => "Confirm your email address to start playing D-Bo!",
    };

    let message = Message::builder()
        .from(MAILBOX.clone())
        .to(player_email.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(variants.txt),
                )
                .multipart(
                    MultiPart::related()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(variants.html),
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

    MAILER.send(message).await?;

    Ok(())
}

// /////// //
// LOCKOUT //
// /////// //

pub async fn send_lockout_email(
    player_email: &str,
    username: &str,
    failed_logins: u8,
    end_lockout: &str,
    language: &LanguagePreference,
) -> DBoResult<()> {
    let helpers = vec![
        PlaceholderHelper::new("{{USERNAME}}", username),
        PlaceholderHelper::new("{{FAILED_LOGINS}}", &format!("{}", failed_logins)),
        PlaceholderHelper::new("{{END_LOCKOUT}}", end_lockout),
        PlaceholderHelper::new("{{BIGDEVDOG_LOGO}}", &ASSETS.images.bigdevdog_logo.cid()),
        PlaceholderHelper::new("{{D_BO_LOGO}}", &ASSETS.images.d_bo_logo.cid()),
    ];

    let variants = replace_placeholders(&ASSETS.templates.lockout.language(language), &helpers);

    let subject = match language {
        LanguagePreference::Spanish => "¡Su cuenta de D-Bo ha sido bloqueado!",
        LanguagePreference::English => "Your D-Bo account has been blocked!",
    };

    let message = Message::builder()
        .from(MAILBOX.clone())
        .to(player_email.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(variants.txt),
                )
                .multipart(
                    MultiPart::related()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(variants.html),
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

    MAILER.send(message).await?;

    Ok(())
}
