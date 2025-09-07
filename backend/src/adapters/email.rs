//! This module is an adapter over the `lettre` crate, allowing for the sending of various types of
//! emails necessary within the application.

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::{
    config::{assets::ASSETS, environment::ENV},
    errors::DBoResult,
};

/// Replace all the placeholders within the confirmation email template with actual values, to send
/// a meaningful and complete email to the player.
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

/// Send a confirmation email to the player.
///
/// ### Arguments
/// - `player_email`: The email address to send to
/// - `username`: The player's username
/// - `token_id`: The confirmation token id
///
/// ### Errors
/// - `ServerSideError`: if the email template files cannot be found.
/// - `AdapterError`: if the email cannot be constructed or sent.
pub async fn send_confirmation_email(
    player_email: &str,
    username: &str,
    token_id: &str,
) -> DBoResult<()> {
    let html_message =
        fill_confirmation_template(&ASSETS.templates.confirmation.html, username, token_id);
    let txt_message =
        fill_confirmation_template(&ASSETS.templates.confirmation.txt, username, token_id);

    let message = Message::builder()
        .from("d-bo@bigdevdog.com".parse().unwrap())
        .to(player_email.parse().unwrap())
        .subject("Confirm your email address to start playing D-Bo")
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
