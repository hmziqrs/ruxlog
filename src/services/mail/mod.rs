use lettre::{AsyncSmtpTransport, AsyncTransport};
mod html_templates;
pub mod smtp;

const DOMAIN: &str = "domain.tld";

async fn send_email(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email_to: &str,
    email_from: &str,
    subject: &str,
    body: String,
) -> Result<(), String> {
    let email_to_parsed = email_to.parse().map_err(|_| "e.to_string()")?;
    let email_from_parsed = email_from.parse().map_err(|_| "e.to_string()")?;
    let email = lettre::Message::builder()
        .from(email_from_parsed)
        .to(email_to_parsed)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| e.to_string())?;

    mailer.send(email).await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn send_email_verification_code(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email: &str,
    code: &str,
) -> Result<(), String> {
    let no_reply = format!("No reply <no-reply@{}>", DOMAIN);
    let subject = "Email verification code";
    let body = html_templates::email_otp_html(code);

    send_email(mailer, email, &no_reply, subject, body).await
}

pub async fn send_forgot_password_email(
    mailer: &AsyncSmtpTransport<lettre::Tokio1Executor>,
    email: &str,
    code: &str,
) -> Result<(), String> {
    let no_reply = format!("No reply <no-reply@{}>", DOMAIN);
    let subject = "Password Reset Verification Code";
    let body = html_templates::email_otp_html(code);

    send_email(mailer, email, &no_reply, subject, body).await
}
