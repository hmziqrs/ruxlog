use dioxus::prelude::*;
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::input::Input;
use oxui::shadcn::label::Label;
use ruxlog_shared::use_auth;

#[component]
pub fn LoginScreen() -> Element {
    let auth_store = use_auth();
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let nav = use_navigator();

    let login_status = auth_store.login_status.read();
    let is_loading = (*login_status).is_loading();

    let handle_submit = move |_| {
        let email_val = email();
        let password_val = password();
        
        spawn(async move {
            let success = auth_store.login(email_val, password_val).await;
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
                    h1 { class: "text-3xl font-bold mb-2", "Welcome back" }
                    p { class: "text-muted-foreground", "Sign in to your account to continue" }
                }

                // Login form
                div { class: "bg-card border border-border rounded-lg p-8 shadow-lg",
                    form {
                        onsubmit: handle_submit,
                        prevent_default: "onsubmit",
                        
                        div { class: "space-y-4",
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
