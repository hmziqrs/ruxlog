use dioxus::prelude::*;

use crate::router::Route;
use crate::screens::auth::{use_register_form, RegisterForm};
use crate::components::MouseTrackingCard;
use ruxlog_shared::store::use_auth;
use oxui::components::animated_grid::{
    AnimatedGridBackground, AnimatedGridCircles, GridContext,
};
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use oxui::components::form::input::AppInput;
use oxui::shadcn::button::Button;

#[component]
pub fn RegisterScreen() -> Element {
    let mut ox_form = use_register_form(RegisterForm::dev());
    let auth_store = use_auth();
    let nav = use_navigator();

    let mut password_match_error = use_signal(|| Option::<String>::None);

    use_context_provider(|| GridContext::new());

    // Navigate to home after successful login (which happens after registration)
    use_effect(move || {
        let login_status = auth_store.login_status.read();
        if login_status.is_success() {
            nav.push(crate::router::Route::HomeScreen {});
        }
    });

    let register_status = auth_store.register_status.read();
    let login_status = auth_store.login_status.read();

    rsx! {
        div { class: "relative flex items-center justify-center min-h-screen overflow-hidden transition-colors duration-300",
            AnimatedGridBackground {}
            AnimatedGridCircles {}
            div { class: "relative z-10 flex w-full justify-center",
                MouseTrackingCard {
                    // Logo or icon placeholder
                    div { class: "flex justify-center mb-2",
                        img {
                            class: "h-26 w-26",
                            src: asset!("/assets/logo.png"),
                            alt: "Logo",
                        }
                    }
                    h1 { class: "text-3xl font-extrabold text-center tracking-tight",
                        "Create Account"
                    }
                    form { class: "space-y-5",
                        onsubmit: |e: Event<FormData>| {
                            e.prevent_default();
                        },
                        AppInput {
                            name: "name",
                            form: ox_form,
                            label: "Full Name",
                            placeholder: "Enter your full name",
                        }
                        AppInput {
                            name: "email",
                            form: ox_form,
                            label: "Email",
                            placeholder: "Enter your email",
                            r#type: "email",
                        }
                        AppInput {
                            name: "password",
                            form: ox_form,
                            label: "Password",
                            placeholder: "Enter your password (min 8 characters)",
                            r#type: "password",
                        }
                        AppInput {
                            name: "confirm_password",
                            form: ox_form,
                            label: "Confirm Password",
                            placeholder: "Confirm your password",
                            r#type: "password",
                        }

                        // Password match error
                        if let Some(error) = password_match_error() {
                            div { class: "p-3 rounded-lg bg-red-500/10 border border-red-500/50 text-sm",
                                "{error}"
                            }
                        }

                        // API error
                        if register_status.is_failed() {
                            ErrorDetails {
                                error: register_status.error.clone(),
                                variant: ErrorDetailsVariant::Minimum,
                                class: "mb-2",
                            }
                        }

                        Button {
                            class: "w-full",
                            disabled: register_status.is_loading() || login_status.is_loading(),
                            onclick: move |e: Event<MouseData>| {
                                e.prevent_default();

                                // Get current form values and check password match
                                let (password, confirm_password) = {
                                    let form_data = ox_form.read();
                                    (form_data.data.password.clone(), form_data.data.confirm_password.clone())
                                };

                                // Clear previous password match error
                                password_match_error.set(None);

                                // Check if passwords match before validation
                                if password != confirm_password {
                                    password_match_error.set(Some("Passwords do not match".to_string()));
                                    return;
                                }

                                ox_form
                                    .write()
                                    .on_submit(move |val| {
                                        let auth_store = auth_store;
                                        let name = val.name.clone();
                                        let email = val.email.clone();
                                        let password = val.password.clone();

                                        spawn(async move {
                                            auth_store.register(name, email, password).await;
                                        });
                                    });
                            },
                            if register_status.is_loading() || login_status.is_loading() {
                                div { class: "loading loading-spinner loading-xs" }
                            }
                            span {
                                if register_status.is_loading() {
                                    "Creating Account..."
                                } else if login_status.is_loading() {
                                    "Logging in..."
                                } else {
                                    "Create Account"
                                }
                            }
                        }
                    }
                    p { class: "text-sm text-center mt-4",
                        "Already have an account? "
                        Link {
                            to: Route::LoginScreen {},
                            class: "font-semibold hover:underline",
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
