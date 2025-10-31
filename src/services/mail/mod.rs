use lettre::{AsyncSmtpTransport, AsyncTransport};
use tracing::{error, info, instrument, warn};

mod html_templates;
pub mod smtp;

const DOMAIN: &str = "domain.tld";

#[instrument(skip(mailer, body), fields(recipient_domain, result))]
async fn send_email(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email_to: &str,
    email_from: &str,
    subject: &str,
    body: String,
) -> Result<(), String> {
    let recipient_domain = email_to.split('@').nth(1).unwrap_or("unknown");
    tracing::Span::current().record("recipient_domain", recipient_domain);

    info!(
        to = %email_to,
        from = %email_from,
        subject = %subject,
        "Sending email"
    );

    let email_to_parsed = email_to.parse().map_err(|e| {
        error!(error = %e, to = %email_to, "Failed to parse recipient email");
        "Invalid recipient email address"
    })?;

    let email_from_parsed = email_from.parse().map_err(|e| {
        error!(error = %e, from = %email_from, "Failed to parse sender email");
        "Invalid sender email address"
    })?;

    let email = lettre::Message::builder()
        .from(email_from_parsed)
        .to(email_to_parsed)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| {
            error!(error = %e, "Failed to build email message");
            e.to_string()
        })?;

    mailer.send(email).await.map_err(|e| {
        error!(error = %e, to = %email_to, "Failed to send email");
        tracing::Span::current().record("result", "failure");
        e.to_string()
    })?;

    info!(to = %email_to, "Email sent successfully");
    tracing::Span::current().record("result", "success");

    Ok(())
}

#[instrument(skip(mailer, code), fields(email_type = "verification"))]
pub async fn send_email_verification_code(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email: &str,
    code: &str,
) -> Result<(), String> {
    info!(to = %email, "Sending email verification code");

    let no_reply = format!("No reply <no-reply@{}>", DOMAIN);
    let subject = "Email verification code";
    let body = html_templates::email_otp_html(code);

    send_email(mailer, email, &no_reply, subject, body).await
}

#[instrument(skip(mailer, code), fields(email_type = "password_reset"))]
pub async fn send_forgot_password_email(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email: &str,
    code: &str,
) -> Result<(), String> {
    info!(to = %email, "Sending password reset email");

    let no_reply = format!("No reply <no-reply@{}>", DOMAIN);
    let subject = "Password Reset Verification Code";
    let body = html_templates::email_otp_html(code);

    send_email(mailer, email, &no_reply, subject, body).await
}
