//! This module holds functions relating to the creation and sending of confirmation emails, which
//! are sent to players during registration, in order to ensure that the email address is valid and
//! can be used to reach that player.

use std::{env, fs::read_to_string};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header},
    transport::smtp::{Error as SmtpError, authentication::Credentials},
};

pub async fn send_confirmation_email(
    player_email: &str,
    username: &str,
    token_id: &str,
) -> Result<(), SmtpError> {
    let frontend_url = env::var("FRONTEND_URL")
        .expect(r#"Environment variable "FRONTEND_URL" is not configured."#);

    let html_template = read_to_string("src/email/templates/confirmation.html").unwrap();

    let html_message = html_template
        .replace("{{USERNAME}}", username)
        .replace("{{TOKEN_ID}}", token_id)
        .replace("{{FRONTEND_URL}}", &frontend_url);
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

    let smtp_username = env::var("SMTP_USERNAME")
        .expect(r#"Environment variable "SMTP_USERNAME" is not configured"#);
    let smtp_password = env::var("SMTP_PASSWORD")
        .expect(r#"Environment variable "SMTP_PASSWORD" is not configured"#);
    let smtp_credentials = Credentials::new(smtp_username, smtp_password);

    let smtp_host =
        env::var("SMTP_HOST").expect(r#"Environment variable "SMTP_HOST" is not configured"#);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
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
