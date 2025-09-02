//! This module holds functions relating to the creation and sending of confirmation emails, which
//! are sent to players during registration, in order to ensure that the email address is valid and
//! can be used to reach that player.

use std::fs::read_to_string;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header},
    transport::smtp::{Error as SmtpError, authentication::Credentials},
};

use crate::environment::ENV;

pub async fn send_confirmation_email(
    player_email: &str,
    username: &str,
    token_id: &str,
) -> Result<(), SmtpError> {
    let html_template = read_to_string("src/email/templates/confirmation.html").unwrap();

    let html_message = html_template
        .replace("{{USERNAME}}", username)
        .replace("{{TOKEN_ID}}", token_id)
        .replace("{{FRONTEND_URL}}", &ENV.frontend_url);
    let message = Message::builder()
        .from("d-bo@bigdevdog.com".parse().unwrap())
        .to(player_email.parse().unwrap())
        .subject("Confirm your email address to start playing D-Bo")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from("This is plaintext")),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_message),
                ),
        )
        .unwrap();

    let smtp_credentials = Credentials::new(ENV.smtp_username.clone(), ENV.smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&ENV.smtp_host)
        .unwrap()
        .credentials(smtp_credentials)
        .build();

    match mailer.send(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}
