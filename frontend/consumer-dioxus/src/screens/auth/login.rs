use dioxus::prelude::*;
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::use_auth;

#[component]
pub fn LoginScreen() -> Element {
    let auth_store = use_auth();
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let nav = use_navigator();

    let login_status = auth_store.login_status.read();
    let is_loading = (*login_status).is_loading();

    let handle_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let email_val = email();
        let password_val = password();
        
        spawn(async move {
            auth_store.login(email_val, password_val).await;
            if auth_store.login_status.read().is_success() {
                nav.push(crate::router::Route::HomeScreen {});
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground flex items-center justify-center p-4",
            div { class: "w-full max-w-md",
                // Header
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold mb-2", "Welcome back" }
                    p { class: "text-muted-foreground", "Sign in to your account to continue" }
                }

                // Login form
                div { class: "bg-card border border-border rounded-lg p-8 shadow-lg",
                    form {
                        onsubmit: handle_submit,
                        
                        div { class: "space-y-4",
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
                            }

                            // Error message
                            if let Some(error) = (*login_status).error_message() {
                                div { class: "p-3 rounded-lg bg-destructive/10 border border-destructive/50 text-destructive text-sm",
                                    "{error}"
                                }
                            }

                            // Submit button
                            Button {
                                r#type: "submit",
                                variant: ButtonVariant::Default,
                                disabled: is_loading,
                                class: "w-full",
                                if is_loading {
                                    "Signing in..."
                                } else {
                                    "Sign in"
                                }
                            }
                        }
                    }

                    // Links
                    div { class: "mt-6 text-center text-sm",
                        span { class: "text-muted-foreground", "Don't have an account? " }
                        a {
                            href: "/register",
                            class: "text-primary hover:underline font-medium",
                            "Sign up"
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
