use dioxus::prelude::*;
use oxcore::http;
use oxstore::StateFrame;
use serde::Serialize;

use crate::screens::auth::{use_register_form, RegisterForm};
use crate::components::MouseTrackingCard;
use ruxlog_shared::store::use_auth;
use oxui::components::animated_grid::{
    AnimatedGridBackground, AnimatedGridCircles, GridContext,
};
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use oxui::components::form::input::AppInput;
use oxui::shadcn::button::Button;

#[derive(Serialize, Clone)]
struct RegisterPayload {
    name: String,
    email: String,
    password: String,
}

#[component]
pub fn RegisterScreen() -> Element {
    let mut ox_form = use_register_form(RegisterForm::dev());
    let auth_store = use_auth();
    let nav = use_navigator();

    let mut register_status = use_signal(|| StateFrame::<()>::new());
    let mut password_match_error = use_signal(|| Option::<String>::None);

    use_context_provider(|| GridContext::new());

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
                    h1 { class: "text-3xl font-extrabold text-center text-zinc-800 dark:text-zinc-100 tracking-tight transition-colors duration-300",
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
                            div { class: "p-3 rounded-lg bg-red-500/10 border border-red-500/50 text-red-600 dark:text-red-400 text-sm",
                                "{error}"
                            }
                        }

                        // API error
                        if register_status.read().is_failed() {
                            ErrorDetails {
                                error: register_status.read().error.clone(),
                                variant: ErrorDetailsVariant::Minimum,
                                class: "mb-2",
                            }
                        }

                        Button {
                            class: "w-full",
                            disabled: register_status.read().is_loading(),
                            onclick: move |e: Event<MouseData>| {
                                e.prevent_default();

                                // Clear password match error
                                password_match_error.set(None);

                                ox_form
                                    .write()
                                    .on_submit(move |val| {
                                        // Check if passwords match
                                        if val.password != val.confirm_password {
                                            password_match_error.set(Some("Passwords do not match".to_string()));
                                            return;
                                        }

                                        let mut register_status = register_status.clone();
                                        let auth_store = auth_store;
                                        let nav = nav.clone();
                                        let email = val.email.clone();
                                        let password = val.password.clone();

                                        spawn(async move {
                                            register_status.write().set_loading();

                                            let payload = RegisterPayload {
                                                name: val.name.clone(),
                                                email: email.clone(),
                                                password: password.clone(),
                                            };

                                            match http::post("/auth/v1/register", &payload).send().await {
                                                Ok(response) => {
                                                    if (200..300).contains(&response.status()) {
                                                        register_status.write().set_success(None);
                                                        // Auto login after successful registration
                                                        auth_store.login(email, password).await;
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
                                    });
                            },
                            if register_status.read().is_loading() {
                                div { class: "loading loading-spinner loading-xs" }
                            }
                            span { "Create Account" }
                        }
                    }
                    p { class: "text-sm text-center text-zinc-600 dark:text-zinc-400 mt-4 transition-colors duration-300",
                        "Already have an account? "
                        a {
                            class: "text-zinc-700 dark:text-zinc-300 font-semibold hover:text-zinc-900 dark:hover:text-zinc-100 transition-colors duration-150",
                            href: "/login",
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
