use std::env;

use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use tracing::{info, instrument};

/// Default to STARTTLS (port 587) unless an implicit-TLS mode is requested.
const TLS_MODE_IMPLICIT: &str = "tls";

/// Resolve the desired TLS mode for the SMTP transport.
///
/// Implicit TLS (SMTPS, port 465) is selected when either:
///   - `SMTP_TLS_MODE=tls` is set, or
///   - `SMTP_PORT=465` is set.
///
/// Otherwise the transport falls back to the existing STARTTLS (port 587)
/// behaviour via `starttls_relay`.
///
/// Reading `SMTP_PORT` here does not change the underlying relay port that
/// lettre selects (`relay`/`starttls_relay` pin 465/587 respectively); it is
/// only consulted to detect the implicit-TLS intent.
fn use_implicit_tls() -> bool {
    if let Ok(mode) = env::var("SMTP_TLS_MODE") {
        if mode.eq_ignore_ascii_case(TLS_MODE_IMPLICIT) {
            return true;
        }
    }
    matches!(env::var("SMTP_PORT").ok().as_deref(), Some("465"))
}

#[instrument(name = "smtp_connection_init")]
pub async fn create_connection() -> AsyncSmtpTransport<Tokio1Executor> {
    let host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let implicit_tls = use_implicit_tls();
    info!(
        smtp_host = %host,
        smtp_user = %username,
        tls_mode = if implicit_tls { "implicit(tls/465)" } else { "starttls(587)" },
        "Initializing SMTP connection"
    );

    let creds = Credentials::new(username, password);

    // Select the transport based on the configured TLS mode:
    //   - implicit TLS (SMTPS, port 465): a full TLS connection is established
    //     up-front via `relay` (Tls::Wrapper). Use this when SMTP_TLS_MODE=tls
    //     or SMTP_PORT=465.
    //   - otherwise: STARTTLS upgrade over a plain connection via
    //     `starttls_relay` (the original behaviour).
    let mailer = if implicit_tls {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .expect("failed to build implicit-TLS SMTP transport")
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .expect("failed to build STARTTLS SMTP transport")
    }
    .credentials(creds)
    .build();

    info!("SMTP connection established");

    mailer
}
