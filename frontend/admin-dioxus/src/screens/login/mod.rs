mod form;
mod mouse_tracking_card;

use dioxus::prelude::*;

use crate::screens::login::form::{use_login_form, LoginForm};
use crate::screens::login::mouse_tracking_card::MouseTrackingCard;
use crate::ui::shadcn::Button;
use crate::{
    components::{
        AnimatedGridBackground, AnimatedGridCircles, AppInput, ErrorDetails, ErrorDetailsVariant,
        GridContext,
    },
    store::use_auth,
};

#[component]
pub fn LoginScreen() -> Element {
    let mut ox_form = use_login_form(LoginForm::dev());
    let auth_store = use_auth();
    let login_status = auth_store.login_status.read();

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
                        "Admin Login"
                    }
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
                        div { class: "flex justify-end text-xs text-zinc-600 dark:text-zinc-400 transition-colors duration-300",
                            a {
                                class: "hover:underline text-zinc-700 dark:text-zinc-300 font-medium hover:text-zinc-900 dark:hover:text-white transition-colors duration-150",
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
                                        });
                                    });
                            },
                            if login_status.is_loading() {
                                div { class: "loading loading-spinner loading-xs" }
                            }
                            span { "Login" }
                        }
                    }
                    p { class: "text-sm text-center text-zinc-600 dark:text-zinc-400 mt-4 transition-colors duration-300",
                        "Don't have an account? "
                        a {
                            class: "text-zinc-700 dark:text-zinc-300 font-semibold hover:text-zinc-900 dark:hover:text-zinc-100 transition-colors duration-150",
                            href: "#",
                            "Sign up"
                        }
                    }
                }
            }
        }
    }
}
