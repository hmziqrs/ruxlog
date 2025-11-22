use std::env;

use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use tracing::{info, instrument};

#[instrument(name = "smtp_connection_init")]
pub async fn create_connection() -> AsyncSmtpTransport<Tokio1Executor> {
    let host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    info!(smtp_host = %host, smtp_user = %username, "Initializing SMTP connection");

    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        .unwrap()
        .credentials(creds)
        .build();

    info!("SMTP connection established");

    mailer
}
