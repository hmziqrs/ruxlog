use dioxus::prelude::*;
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::input::Input;
use oxui::shadcn::label::Label;
use ruxlog_shared::use_auth;

#[component]
pub fn RegisterScreen() -> Element {
    let auth_store = use_auth();
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut validation_error = use_signal(|| Option::<String>::None);
    let nav = use_navigator();

    let register_status = auth_store.register_status.read();
    let is_loading = (*register_status).is_loading();

    let handle_submit = move |_| {
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

        spawn(async move {
            let success = auth_store.register(name_val, email_val, password_val).await;
            if success {
                nav.push(crate::router::Route::HomeScreen {});
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
                        prevent_default: "onsubmit",
                        
                        div { class: "space-y-4",
                            // Name field
                            div { class: "space-y-2",
                                Label { r#for: "name", "Full Name" }
                                Input {
                                    id: "name",
                                    r#type: "text",
                                    placeholder: "John Doe",
                                    value: "{name}",
                                    oninput: move |evt| name.set(evt.value().clone()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }

                            // Email field
                            div { class: "space-y-2",
                                Label { r#for: "email", "Email" }
                                Input {
                                    id: "email",
                                    r#type: "email",
                                    placeholder: "you@example.com",
                                    value: "{email}",
                                    oninput: move |evt| email.set(evt.value().clone()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }

                            // Password field
                            div { class: "space-y-2",
                                Label { r#for: "password", "Password" }
                                Input {
                                    id: "password",
                                    r#type: "password",
                                    placeholder: "••••••••",
                                    value: "{password}",
                                    oninput: move |evt| password.set(evt.value().clone()),
                                    required: true,
                                    disabled: is_loading,
                                }
                                p { class: "text-xs text-muted-foreground", "Must be at least 8 characters" }
                            }

                            // Confirm password field
                            div { class: "space-y-2",
                                Label { r#for: "confirm_password", "Confirm Password" }
                                Input {
                                    id: "confirm_password",
                                    r#type: "password",
                                    placeholder: "••••••••",
                                    value: "{confirm_password}",
                                    oninput: move |evt| confirm_password.set(evt.value().clone()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }

                            // Error message
                            if let Some(error) = validation_error() {
                                div { class: "p-3 rounded-lg bg-destructive/10 border border-destructive/50 text-destructive text-sm",
                                    "{error}"
                                }
                            } else if let Some(error) = (*register_status).error_message() {
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
