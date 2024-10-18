use std::env;

use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};

pub async fn create_connection() -> AsyncSmtpTransport<Tokio1Executor> {
    let host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    // let port = env::var("SMTP_PORT")
    //     .expect("SMTP_PORT must be set")
    //     .parse::<u16>()
    //     .expect("Invalid SMTP_PORT");
    let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        .unwrap()
        .credentials(creds)
        // .port(port)
        .build();

    mailer
}
