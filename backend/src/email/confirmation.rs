//! This module holds functions relating to the creation and sending of confirmation emails, which
//! are sent to players during registration, in order to ensure that the email address is valid and
//! can be used to reach that player.

use std::{env, fs::read_to_string};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header},
    transport::smtp::{Error as SmtpError, authentication::Credentials},
};

pub async fn send_confirmation_email(player_email: &str, username: &str) -> Result<(), SmtpError> {
    let html_template = read_to_string("src/email/templates/confirmation.html").unwrap();

    let html_message = html_template.replace("{{USERNAME}}", username);

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
    // .body(String::from(
    //     "Click the link to confirm your email address.",
    // ))
    // .unwrap();

    let username = env::var("SMTP_USERNAME")
        .expect(r#"Environment variable "SMTP_USERNAME" is not configured"#);
    let password = env::var("SMTP_PASSWORD")
        .expect(r#"Environment variable "SMTP_PASSWORD" is not configured"#);
    let credentials = Credentials::new(username, password);

    let host =
        env::var("SMTP_HOST").expect(r#"Environment variable "SMTP_HOST" is not configured"#);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        .unwrap()
        .credentials(credentials)
        .build();

    match mailer.send(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}
