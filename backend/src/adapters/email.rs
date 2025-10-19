//! This module is an adapter over the `lettre` crate, allowing for the sending of various types of
//! emails necessary within the application.

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    config::{
        assets::{ASSETS, EmailLocalizationVariants},
        environment::ENV,
    },
    errors::{DBoError, DBoResult},
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

/// A struct containing information related to **value placeholders** (like `"{{USERNAME}}"`)
struct PlaceholderHelper {
    /// The placeholder that should be replaced by the value in a formatted email.
    placeholder: &'static str,
    /// The value that should replace the placeholder in a formatted email.
    value: String,
}

impl PlaceholderHelper {
    pub fn new(placeholder: &'static str, value: &str) -> Self {
        Self {
            placeholder: placeholder,
            value: String::from(value),
        }
    }

    pub fn username(value: &str) -> Self {
        Self::new("{{USERNAME}}", value)
    }

    pub fn player_id(value: &str) -> Self {
        Self::new("{{PLAYER_ID}}", value)
    }

    pub fn token_id(value: &str) -> Self {
        Self::new("{{TOKEN_ID}}", value)
    }

    pub fn conf_token_id(value: &str) -> Self {
        Self::new("{{CONF_TOKEN_ID}}", value)
    }

    pub fn undo_token_id(value: &str) -> Self {
        Self::new("{{UNDO_TOKEN_ID}}", value)
    }

    pub fn old_email(value: &str) -> Self {
        Self::new("{{OLD_EMAIL}}", value)
    }

    pub fn new_email(value: &str) -> Self {
        Self::new("{{NEW_EMAIL}}", value)
    }

    pub fn frontend_url() -> Self {
        Self::new("{{FRONTEND_URL}}", &ENV.frontend_url)
    }
}

/// Replace all value placeholders in a template with their proper values. Make sure to include all
/// placeholders in `helpers`, or else the email will not be formatted properly.
///
/// ### Arguments
/// - `template`: The template to fill in
/// - `helpers`: The PlaceholderHelpers indicating the placeholders to search for as well as the
///   values that should replace them.
fn replace_placeholders(template: &str, helpers: &Vec<PlaceholderHelper>) -> String {
    let mut value = String::from(template);

    for helper in helpers {
        value = value.replace(helper.placeholder, &helper.value);
    }

    value
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

pub fn format_date_time(
    utc_time: &DateTime<Utc>,
    language: &LanguagePreference,
    time_zone_str: &str,
) -> DBoResult<String> {
    let tz: Tz = time_zone_str.parse()?;

    let local = utc_time.with_timezone(&tz);

    let weekday = match (language, local.weekday()) {
        (LanguagePreference::English, Weekday::Sun) => "Sunday",
        (LanguagePreference::English, Weekday::Mon) => "Monday",
        (LanguagePreference::English, Weekday::Tue) => "Tuesday",
        (LanguagePreference::English, Weekday::Wed) => "Wednesday",
        (LanguagePreference::English, Weekday::Thu) => "Thursday",
        (LanguagePreference::English, Weekday::Fri) => "Friday",
        (LanguagePreference::English, Weekday::Sat) => "Saturday",

        (LanguagePreference::Spanish, Weekday::Sun) => "Domingo",
        (LanguagePreference::Spanish, Weekday::Mon) => "Lunes",
        (LanguagePreference::Spanish, Weekday::Tue) => "Martes",
        (LanguagePreference::Spanish, Weekday::Wed) => "Miércoles",
        (LanguagePreference::Spanish, Weekday::Thu) => "Jueves",
        (LanguagePreference::Spanish, Weekday::Fri) => "Viernes",
        (LanguagePreference::Spanish, Weekday::Sat) => "Sábado",
    };

    let formatter = match language {
        LanguagePreference::English => "%m/%d/%Y at %I:%M:%S %P",
        LanguagePreference::Spanish => match local.hour() == 1 {
            false => "%d/%m/%Y a las %H:%M:%S",
            true => "%d/%m/%Y a la %H:%M:%S",
        },
    };

    let formatted_date_time = local.format(formatter).to_string();

    Ok(format!("{}, {}", weekday, formatted_date_time))
}

/// Build a branded message from an email template. This function takes all information that may be
/// needed in order to fill in the templates correctly. It will replace all provided placeholders in
/// the email templates - both value placeholders (like `"{{USERNAME}}"`) as well as gendered
/// placeholders (like `"**<m>/<f>/<nb>**`). It will construct a multi-part Message, with one part
/// being the plaintext message, and the other part containing the HTML message, alongside both the
/// D-Bo logo and the BigDevDog logo.
///
/// The function will automatically add the PlaceholderHelper for the CIDs within the HTML template.
/// **Do not** include these within the `helpers` argument, as it will just slow the function down.
///
/// ### Arguments
/// - `to`: The email address that the message will be sent to.
/// - `templates`: The type of email to be sent.
/// - `language`: The language that the email will be sent in.
/// - `helpers`: The value placeholders that should be replaced in the templates. Again, this should
///   **not** include the placeholders for the D-Bo logo and BigDevDog logo CIDs.
/// - `gender`: The gender of the player receiving this message. This is **always** ignored for
///   messages sent in English. If the value is None for Spanish messages, the gendered placeholders
///   will **not** be replaced. This is preferred for messages not including gendered placeholders,
///   as it will make the function faster, but use caution.
///
/// ### Errors
/// - `InvalidEmailAddress` if the **to** argument cannot be parsed into a Mailbox.
/// - `AdapterError` if the message cannot be constructed.
fn build_branded_message(
    to: &str,
    templates: &EmailLocalizationVariants,
    language: &LanguagePreference,
    helpers: &mut Vec<PlaceholderHelper>,
    gender: &Option<Gender>,
) -> DBoResult<Message> {
    let message_info = templates.language(language);
    let txt = match language {
        LanguagePreference::English => replace_placeholders(&message_info.txt, helpers),
        LanguagePreference::Spanish => {
            let intermediate = match gender {
                Some(g) => fill_gendered_template(&message_info.txt, g),
                None => message_info.txt.clone(),
            };
            replace_placeholders(&intermediate, helpers)
        }
    };

    helpers.push(PlaceholderHelper::new(
        "{{D_BO_LOGO}}",
        &ASSETS.images.bigdevdog_logo.cid(),
    ));
    helpers.push(PlaceholderHelper::new(
        "{{BIGDEVDOG_LOGO}}",
        &ASSETS.images.bigdevdog_logo.cid(),
    ));

    let html = match language {
        LanguagePreference::English => replace_placeholders(&message_info.html, helpers),
        LanguagePreference::Spanish => {
            let intermediate = match gender {
                Some(g) => fill_gendered_template(&message_info.html, g),
                None => message_info.html.clone(),
            };
            replace_placeholders(&intermediate, helpers)
        }
    };

    let to_mailbox: Mailbox = match to.parse() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("CRITICAL ERROR ENCOUNTERED!");
            eprintln!("Email failed to send due to invalid recipient mailbox!");
            eprintln!("Invalid address: {}", to);
            eprintln!("Error Debug: {:?}", e);
            return Err(DBoError::InvalidEmailAddress);
        }
    };

    Ok(Message::builder()
        .from(MAILBOX.clone())
        .to(to_mailbox)
        .subject(message_info.subject.clone())
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(txt),
                )
                .multipart(
                    MultiPart::related()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html),
                        )
                        .singlepart(
                            Attachment::new_inline(ASSETS.images.bigdevdog_logo.cid()).body(
                                ASSETS.images.bigdevdog_logo.bytes(),
                                ASSETS.images.bigdevdog_logo.mime_type(),
                            ),
                        )
                        .singlepart(Attachment::new_inline(ASSETS.images.d_bo_logo.cid()).body(
                            ASSETS.images.d_bo_logo.bytes(),
                            ASSETS.images.d_bo_logo.mime_type(),
                        )),
                ),
        )?)
}

// ///////////// //
// EMAIL SENDERS //
// ///////////// //

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
/// - `InvalidEmailAddress` if the **player_email** argument cannot be parsed into a Mailbox.
/// - `AdapterError` if the email cannot be constructed or sent.
pub async fn send_registration_email(
    player_email: &str,
    username: &str,
    token_id: &str,
    player_id: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::username(username),
        PlaceholderHelper::frontend_url(),
        PlaceholderHelper::token_id(token_id),
        PlaceholderHelper::player_id(player_id),
    ];

    let message = build_branded_message(
        player_email,
        &ASSETS.templates.registration,
        language,
        &mut helpers,
        &Some(pronoun.clone()),
    )?;

    MAILER.send(message).await?;

    Ok(())
}

/// Send a lockout email, informing a player that their account has been locked from logging in due
/// to a five or more failed login attempts.
///
/// ### Arguments
/// - `player_email`: The email address to send the message to
/// - `username`: The player's username
/// - `failed_logins`: The number of failed logins resulting in this lockout
/// - `end_lockout`: The time at which their lockout will be over
/// - `language`: The language to send the email in
///
/// ### Errors
/// - `InvalidEmailAddress` if the player_email cannot be parsed into a Mailbox.
/// - `TimeZoneParseError` if the time zone string cannot be parsed.
/// - `AdapterError` if the message cannot be constructed or sent.
pub async fn send_lockout_email(
    player_email: &str,
    username: &str,
    failed_logins: u8,
    end_lockout: &DateTime<Utc>,
    time_zone_str: &str,
    language: &LanguagePreference,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::username(username),
        PlaceholderHelper::new("{{FAILED_LOGINS}}", &format!("{}", failed_logins)),
        PlaceholderHelper::new(
            "{{END_LOCKOUT}}",
            &format_date_time(end_lockout, language, time_zone_str)?,
        ),
    ];

    let message = build_branded_message(
        player_email,
        &ASSETS.templates.lockout,
        language,
        &mut helpers,
        &None,
    )?;

    MAILER.send(message).await?;

    Ok(())
}

/// Send an email to the player's newly proposed email address, providing them with a link to
/// confirm their new mailbox. It also includes a link to **undo** the operation, if the email was
/// sent to them by mistake.
///
/// ### Arguments
/// - `username`: The player's username
/// - `old_email`: The player's current email address
/// - `new_email`: The player's newly proposed email address, to which this email is sent
/// - `player_id`: The player's unique identifier
/// - `conf_token_id`: The ID of the confirmation token used to confirm the change
/// - `undo_token_id`: The ID of the undo token used to revert the change
/// - `language`: The language to send the email in
/// - `pronoun`: The player's preferred pronouns, for Spanish emails
///
/// ### Errors
/// - `InvalidEmailAddress` if the new email cannot be parsed into a mailbox
/// - `AdapterError` if the email cannot be constructed or sent due to a server-side error
pub async fn send_change_email_confirmation_email(
    username: &str,
    old_email: &str,
    new_email: &str,
    player_id: &str,
    conf_token_id: &str,
    undo_token_id: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::username(username),
        PlaceholderHelper::old_email(old_email),
        PlaceholderHelper::new_email(new_email),
        PlaceholderHelper::frontend_url(),
        PlaceholderHelper::player_id(player_id),
        PlaceholderHelper::conf_token_id(conf_token_id),
        PlaceholderHelper::undo_token_id(undo_token_id),
    ];

    let message = build_branded_message(
        new_email,
        &ASSETS.templates.change_email_confirmation,
        language,
        &mut helpers,
        &Some(pronoun.clone()),
    )?;

    MAILER.send(message).await?;

    Ok(())
}

/// Send a warning email to a player's current confirmed email address, informing them that a
/// request has been made to change their email address. This email provides them with a link
/// allowing them to undo the change if they did not request this.
///
/// ### Arguments
/// - `username`: The player's username
/// - `old_email`: The player's current email address, to which this email is sent
/// - `new_email`: The player's newly proposed email address
/// - `player_id`: The player's unique identifier
/// - `undo_token_id`: The undo token's unique identifier
/// - `language`: The language to send the email in
///
/// ### Errors
/// - `InvalidEmailAddress` if the old email cannot be parsed into a Mailbox
/// - `AdapterError` if the message cannot be constructed or sent due to a server-side error
pub async fn send_change_email_warning_email(
    username: &str,
    old_email: &str,
    new_email: &str,
    player_id: &str,
    undo_token_id: &str,
    language: &LanguagePreference,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::username(username),
        PlaceholderHelper::old_email(old_email),
        PlaceholderHelper::new_email(new_email),
        PlaceholderHelper::frontend_url(),
        PlaceholderHelper::player_id(player_id),
        PlaceholderHelper::undo_token_id(undo_token_id),
    ];

    let message = build_branded_message(
        old_email,
        &ASSETS.templates.change_email_warning,
        language,
        &mut helpers,
        &None,
    )?;

    MAILER.send(message).await?;

    Ok(())
}

/// Send an email to the player informing them that their password has been changed. This email
/// provides them with a link to reset their password securely if this was done by mistake.
///
/// ### Arguments
/// - `player_email`: The player's email address
/// - `username`: The player's username
/// - `player_id`: The player's unique identifier
/// - `undo_token_id`: The undo token's unique identifier
/// - `language`: The language to send the email in
/// - `pronoun`: The player's preferred pronouns, for valid Spanish emails
///
/// ### Errors
/// - `InvalidEmailAddress` if the player's email address cannot be parsed into a Mailbox
/// - `AdapterError` if the message cannot be constructed or sent due to a server-side error
pub async fn send_change_password_email(
    player_email: &str,
    username: &str,
    player_id: &str,
    undo_token_id: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::username(username),
        PlaceholderHelper::frontend_url(),
        PlaceholderHelper::player_id(player_id),
        PlaceholderHelper::undo_token_id(undo_token_id),
    ];

    let message = build_branded_message(
        player_email,
        &ASSETS.templates.change_password,
        language,
        &mut helpers,
        &Some(pronoun.clone()),
    )?;

    MAILER.send(message).await?;

    Ok(())
}

/// Send an email to a player informing them that their username has been changed.
///
/// ### Arguments
/// - `player_email`: The email to send the message to
/// - `old_username`: The player's old username
/// - `new_username`: The player's new username
/// - `language`: The language to send the email in
/// - `pronoun`: The player's preferred pronouns, for valid Spanish emails
///
/// ### Errors
/// - `InvalidEmailAddress` if the player email cannot be parsed into a Mailbox
/// - `AdapterError` if the message cannot be constructed or sent due to a server-side error
pub async fn send_change_username_email(
    player_email: &str,
    old_username: &str,
    new_username: &str,
    language: &LanguagePreference,
    pronoun: &Gender,
) -> DBoResult<()> {
    let mut helpers = vec![
        PlaceholderHelper::new("{{NEW_USERNAME}}", new_username),
        PlaceholderHelper::new("{{OLD_USERNAME}}", old_username),
    ];

    let message = build_branded_message(
        player_email,
        &ASSETS.templates.change_username,
        language,
        &mut helpers,
        &Some(pronoun.clone()),
    )?;

    MAILER.send(message).await?;

    Ok(())
}
