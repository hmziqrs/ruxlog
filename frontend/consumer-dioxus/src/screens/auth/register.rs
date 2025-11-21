use dioxus::prelude::*;
use oxcore::http;
use oxstore::StateFrame;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::use_auth;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct RegisterPayload {
    name: String,
    email: String,
    password: String,
}

#[component]
pub fn RegisterScreen() -> Element {
    let auth_store = use_auth();
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut validation_error = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let register_status = use_signal(|| StateFrame::<()>::new());
    let is_loading = register_status.read().is_loading();

    let handle_submit = move |evt: FormEvent| {
        evt.prevent_default();
        // Client-side validation
        let name_val = name();
        let email_val = email();
        let password_val = password();
        let confirm_password_val = confirm_password();

        if name_val.trim().is_empty() {
            validation_error.set(Some("Name is required".to_string()));
            return;
        }

        if email_val.trim().is_empty() {
            validation_error.set(Some("Email is required".to_string()));
            return;
        }

        if password_val.len() < 8 {
            validation_error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }

        if password_val != confirm_password_val {
            validation_error.set(Some("Passwords do not match".to_string()));
            return;
        }

        validation_error.set(None);

        let mut register_status = register_status.clone();
        let auth_store = auth_store;
        let nav = nav.clone();

        spawn(async move {
            register_status.write().set_loading();

            let payload = RegisterPayload {
                name: name_val.clone(),
                email: email_val.clone(),
                password: password_val.clone(),
            };

            match http::post("/auth/v1/register", &payload).send().await {
                Ok(response) => {
                    if (200..300).contains(&response.status()) {
                        register_status.write().set_success(None);
                        auth_store.login(email_val, password_val).await;
                        if auth_store.login_status.read().is_success() {
                            nav.push(crate::router::Route::HomeScreen {});
                        }
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        register_status.write().set_api_error(status, body);
                    }
                }
                Err(e) => {
                    let (kind, msg) = oxstore::error::classify_transport_error(&e);
                    register_status
                        .write()
                        .set_transport_error(kind, Some(msg));
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground flex items-center justify-center p-4",
            div { class: "w-full max-w-md",
                // Header
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold mb-2", "Create account" }
                    p { class: "text-muted-foreground", "Join our community today" }
                }

                // Register form
                div { class: "bg-card border border-border rounded-lg p-8 shadow-lg",
                    form {
                        onsubmit: handle_submit,
                        
                        div { class: "space-y-4",
                            // Name field
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", r#for: "name", "Full Name" }
                                SimpleInput {
                                    id: Some("name".to_string()),
                                    placeholder: Some("John Doe".to_string()),
                                    value: name(),
                                    oninput: move |value| name.set(value),
                                    disabled: is_loading,
                                }
                            }

                            // Email field
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", r#for: "email", "Email" }
                                SimpleInput {
                                    id: Some("email".to_string()),
                                    r#type: "email".to_string(),
                                    placeholder: Some("you@example.com".to_string()),
                                    value: email(),
                                    oninput: move |value| email.set(value),
                                    disabled: is_loading,
                                }
                            }

                            // Password field
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", r#for: "password", "Password" }
                                SimpleInput {
                                    id: Some("password".to_string()),
                                    r#type: "password".to_string(),
                                    placeholder: Some("••••••••".to_string()),
                                    value: password(),
                                    oninput: move |value| password.set(value),
                                    disabled: is_loading,
                                }
                                p { class: "text-xs text-muted-foreground", "Must be at least 8 characters" }
                            }

                            // Confirm password field
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", r#for: "confirm_password", "Confirm Password" }
                                SimpleInput {
                                    id: Some("confirm_password".to_string()),
                                    r#type: "password".to_string(),
                                    placeholder: Some("••••••••".to_string()),
                                    value: confirm_password(),
                                    oninput: move |value| confirm_password.set(value),
                                    disabled: is_loading,
                                }
                            }

                            // Error messages
                            if let Some(error) = validation_error() {
                                div { class: "p-3 rounded-lg bg-destructive/10 border border-destructive/50 text-destructive text-sm",
                                    "{error}"
                                }
                            }

                            if register_status.read().is_failed() {
                                ErrorDetails {
                                    error: register_status.read().error.clone(),
                                    variant: ErrorDetailsVariant::Minimum,
                                    class: "mb-2",
                                }
                            }

                            // Submit button
                            Button {
                                r#type: "submit",
                                variant: ButtonVariant::Default,
                                disabled: is_loading,
                                class: "w-full",
                                if is_loading {
                                    "Creating account..."
                                } else {
                                    "Create account"
                                }
                            }
                        }
                    }

                    // Links
                    div { class: "mt-6 text-center text-sm",
                        span { class: "text-muted-foreground", "Already have an account? " }
                        a {
                            href: "/login",
                            class: "text-primary hover:underline font-medium",
                            "Sign in"
                        }
                    }
                }

                // Footer
                div { class: "mt-8 text-center text-sm text-muted-foreground",
                    "By continuing, you agree to our Terms of Service and Privacy Policy"
                }
            }
        }
    }
}
