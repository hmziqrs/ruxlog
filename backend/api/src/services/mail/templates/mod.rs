use tera::Tera;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Failed to render template '{name}': {source}")]
    Render {
        name: String,
        #[source]
        source: tera::Error,
    },
}

/// Lazy-initialized Tera instance with all email templates.
fn tera_instance() -> Result<Tera, TemplateError> {
    let mut tera = Tera::default();

    tera.add_raw_template(
        "email_verification",
        include_str!("email_verification.html"),
    )
    .map_err(|e| TemplateError::Render {
        name: "email_verification".into(),
        source: e,
    })?;

    tera.add_raw_template("forgot_password", include_str!("forgot_password.html"))
        .map_err(|e| TemplateError::Render {
            name: "forgot_password".into(),
            source: e,
        })?;

    tera.add_raw_template("welcome", include_str!("welcome.html"))
        .map_err(|e| TemplateError::Render {
            name: "welcome".into(),
            source: e,
        })?;

    tera.add_raw_template(
        "newsletter_confirmation",
        include_str!("newsletter_confirmation.html"),
    )
    .map_err(|e| TemplateError::Render {
        name: "newsletter_confirmation".into(),
        source: e,
    })?;

    tera.add_raw_template("payment_receipt", include_str!("payment_receipt.html"))
        .map_err(|e| TemplateError::Render {
            name: "payment_receipt".into(),
            source: e,
        })?;

    tera.add_raw_template(
        "subscription_confirmation",
        include_str!("subscription_confirmation.html"),
    )
    .map_err(|e| TemplateError::Render {
        name: "subscription_confirmation".into(),
        source: e,
    })?;

    Ok(tera)
}

/// Render a named email template with the given context.
///
/// # Template names
///
/// - `"email_verification"` — variables: `app_name`, `verification_url`, `user_name`
/// - `"forgot_password"` — variables: `app_name`, `reset_url`, `user_name`
/// - `"welcome"` — variables: `app_name`, `user_name`, `login_url`
/// - `"newsletter_confirmation"` — variables: `app_name`, `confirm_url`
/// - `"payment_receipt"` — variables: `app_name`, `user_name`, `amount`, `currency`, `plan_name`, `invoice_url`
/// - `"subscription_confirmation"` — variables: `app_name`, `user_name`, `plan_name`, `amount`, `next_billing_date`
///
/// All templates also support an optional `primary_color` variable (defaults to `"#3b82f6"`).
pub fn render(template_name: &str, context: &tera::Context) -> Result<String, TemplateError> {
    let tera = tera_instance()?;

    tera.render(template_name, context)
        .map_err(|e| TemplateError::Render {
            name: template_name.into(),
            source: e,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_email_verification() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert("user_name", "Alice");
        ctx.insert(
            "verification_url",
            "https://example.com/verify?token=abc123",
        );
        let html = render("email_verification", &ctx).unwrap();
        assert!(html.contains("Alice"));
        assert!(html.contains("TestApp"));
        assert!(html.contains("https://example.com/verify?token=abc123"));
    }

    #[test]
    fn render_forgot_password() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert("user_name", "Bob");
        ctx.insert("reset_url", "https://example.com/reset?token=xyz789");
        let html = render("forgot_password", &ctx).unwrap();
        assert!(html.contains("Bob"));
        assert!(html.contains("https://example.com/reset?token=xyz789"));
    }

    #[test]
    fn render_welcome() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert("user_name", "Carol");
        ctx.insert("login_url", "https://example.com/login");
        let html = render("welcome", &ctx).unwrap();
        assert!(html.contains("Carol"));
        assert!(html.contains("Welcome"));
    }

    #[test]
    fn render_newsletter_confirmation() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert(
            "confirm_url",
            "https://example.com/newsletter/confirm?token=t",
        );
        let html = render("newsletter_confirmation", &ctx).unwrap();
        assert!(html.contains("Confirm Subscription"));
    }

    #[test]
    fn render_payment_receipt() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert("user_name", "Dave");
        ctx.insert("amount", "9.99");
        ctx.insert("currency", "$");
        ctx.insert("plan_name", "Pro");
        ctx.insert("invoice_url", "https://example.com/invoice/42");
        let html = render("payment_receipt", &ctx).unwrap();
        assert!(html.contains("$9.99"));
        assert!(html.contains("Pro"));
    }

    #[test]
    fn render_subscription_confirmation() {
        let mut ctx = tera::Context::new();
        ctx.insert("app_name", "TestApp");
        ctx.insert("user_name", "Eve");
        ctx.insert("plan_name", "Enterprise");
        ctx.insert("amount", "$29.00/mo");
        ctx.insert("next_billing_date", "2026-06-12");
        let html = render("subscription_confirmation", &ctx).unwrap();
        assert!(html.contains("Enterprise"));
        assert!(html.contains("2026-06-12"));
    }

    #[test]
    fn unknown_template_returns_error() {
        let ctx = tera::Context::new();
        let result = render("nonexistent", &ctx);
        assert!(result.is_err());
    }
}
