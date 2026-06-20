use dioxus::prelude::*;

use crate::components::MouseTrackingCard;
use crate::router::Route;
use crate::screens::auth::{use_login_form, LoginForm};
use oxui::components::animated_grid::{AnimatedGridBackground, AnimatedGridCircles, GridContext};
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use oxui::components::form::input::AppInput;
use oxui::shadcn::button::Button;
use ruxlog_shared::store::use_auth;

#[component]
pub fn LoginScreen() -> Element {
    let mut ox_form = use_login_form(LoginForm::dev());
    let auth_store = use_auth();
    let login_status = auth_store.login_status.read();
    let login_totp = auth_store.login_totp.read();
    // F#4/F#7/F#16: a pending TOTP step is present when `login_totp` carries
    // the opaque token the server returned for a 2FA-enrolled user. While it
    // is set we render a code input instead of the password form.
    let pending_totp_token = login_totp.data.clone();
    let nav = use_navigator();

    use_context_provider(GridContext::new);

    let signup_link = rsx! {
        p { class: "text-sm text-center mt-4",
            "Don't have an account? "
            Link {
                to: Route::RegisterScreen {},
                class: "font-semibold hover:underline",
                "Sign up"
            }
        }
    };

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
                        "Consumer Login"
                    }
                    // Two-step 2FA-at-login (F#4/F#7/F#16): a correct password
                    // is not enough for a 2FA-enrolled user. Show a TOTP code
                    // input and POST it (with the pending token) to the second
                    // login step.
                    if let Some(totp_token) = pending_totp_token {
                        form { class: "space-y-5",
                            onsubmit: |e: Event<FormData>| {
                                e.prevent_default();
                            },
                            p { class: "text-sm text-center",
                                "Enter the 6-digit code from your authenticator app."
                            }
                            AppInput {
                                name: "totp_code",
                                form: ox_form,
                                label: "Authentication code",
                                placeholder: "123456",
                            }
                            if login_status.is_failed() {
                                ErrorDetails {
                                    error: login_status.error.clone(),
                                    variant: ErrorDetailsVariant::Minimum,
                                    class: "mb-2",
                                }
                            }
                            Button {
                                class: "w-full",
                                disabled: login_status.is_loading(),
                                onclick: move |e: Event<MouseData>| {
                                    e.prevent_default();
                                    let token = totp_token.clone();
                                    ox_form
                                        .write()
                                        .on_submit(move |val| {
                                            let code = val.totp_code.clone();
                                            let token = token.clone();
                                            spawn(async move {
                                                auth_store.verify_login_totp(token, code).await;
                                                if auth_store.login_status.read().is_success() {
                                                    nav.push(crate::router::Route::HomeScreen {});
                                                }
                                            });
                                        });
                                },
                                if login_status.is_loading() {
                                    div { class: "loading loading-spinner loading-xs" }
                                }
                                span { "Verify" }
                            }
                        }
                    } else {
                        form { class: "space-y-5",
                            onsubmit: |e: Event<FormData>| {
                                e.prevent_default();
                            },
                            AppInput {
                                name: "email",
                                form: ox_form,
                                label: "Email",
                                placeholder: "Enter your email",
                            }
                            AppInput {
                                name: "password",
                                form: ox_form,
                                label: "Password",
                                placeholder: "Enter your password",
                                r#type: "password",
                            }
                            if login_status.is_failed() {
                                ErrorDetails {
                                    error: login_status.error.clone(),
                                    variant: ErrorDetailsVariant::Minimum,
                                    class: "mb-2",
                                }
                            }
                            div { class: "flex justify-end text-xs",
                                a {
                                    class: "hover:underline font-medium",
                                    href: "#",
                                    "Forgot password?"
                                }
                            }
                            Button {
                                class: "w-full",
                                disabled: login_status.is_loading(),
                                onclick: move |e: Event<MouseData>| {
                                    e.prevent_default();
                                    ox_form
                                        .write()
                                        .on_submit(move |val| {
                                            spawn(async move {
                                                let email = val.email.clone();
                                                let password = val.password.clone();
                                                auth_store.login(email, password).await;
                                                if auth_store.login_status.read().is_success()
                                                    && auth_store.login_totp.read().data.is_none()
                                                {
                                                    nav.push(crate::router::Route::HomeScreen {});
                                                }
                                            });
                                        });
                                },
                                if login_status.is_loading() {
                                    div { class: "loading loading-spinner loading-xs" }
                                }
                                span { "Login" }
                            }
                        }
                    }
                    { signup_link }
                }
            }
        }
    }
}
