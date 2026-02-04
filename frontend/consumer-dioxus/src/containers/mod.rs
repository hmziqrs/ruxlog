use crate::config::{DarkMode, BRAND};
use crate::router::Route;
use crate::utils::persist;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::{
    ld_icons::{LdGithub, LdMoon, LdSun},
    si_icons::SiX,
};

use hmziq_dioxus_free_icons::Icon;

#[cfg(feature = "consumer-auth")]
use hmziq_dioxus_free_icons::icons::ld_icons::LdLogIn;
#[cfg(feature = "consumer-auth")]
use ruxlog_shared::use_auth;

pub mod auth_guard_wrapper;
pub use auth_guard_wrapper::*;

#[component]
pub fn NavBarContainer() -> Element {
    #[cfg(feature = "consumer-auth")]
    let auth_store = use_auth();
    #[cfg(feature = "consumer-auth")]
    let user = auth_store.user.read();

    let mut dark_theme = use_context_provider(|| Signal::new(DarkMode(true)));

    // Initialize theme from DOM
    use_effect(move || {
        spawn(async move {
            let is_dark =
                document::eval("return document.documentElement.classList.contains('dark');")
                    .await
                    .unwrap()
                    .to_string();
            dark_theme.set(DarkMode(is_dark.parse::<bool>().unwrap_or(false)));
        });
    });

    let toggle_dark_mode = move |_: MouseEvent| {
        dark_theme.write().toggle();
        let is_dark = (*dark_theme.read()).0;
        spawn(async move {
            _ = document::eval("document.documentElement.classList.toggle('dark');").await;
        });
        persist::set_theme(if is_dark { "dark" } else { "light" });
    };

    // Prepare auth UI element (conditionally compiled)
    let auth_ui: Option<Element> = {
        #[cfg(feature = "consumer-auth")]
        {
            if let Some(user) = &*user {
                #[cfg(feature = "profile-management")]
                {
                    Some(rsx! {
                        Link {
                            to: Route::ProfileScreen {},
                            class: "flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted/80 transition-all duration-200 active:scale-95",
                            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-semibold text-sm",
                                "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            span { class: "hidden md:block text-sm font-medium", "{user.name}" }
                        }
                    })
                }
                #[cfg(not(feature = "profile-management"))]
                {
                    Some(rsx! {
                        div {
                            class: "flex items-center gap-2 px-3 py-2",
                            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-semibold text-sm",
                                "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            span { class: "hidden md:block text-sm font-medium", "{user.name}" }
                        }
                    })
                }
            } else {
                Some(rsx! {
                    Link {
                        to: Route::LoginScreen {},
                        class: "icon-button",
                        aria_label: "Sign In",
                        Icon { icon: LdLogIn, class: "w-5 h-5" }
                    }
                })
            }
        }
        #[cfg(not(feature = "consumer-auth"))]
        {
            None
        }
    };

    rsx! {
        div { class: "min-h-screen bg-background flex flex-col",
            // Navbar
            nav { class: "navbar-container",
                div { class: "container mx-auto px-4 max-w-6xl",
                    div { class: "flex h-16 items-center justify-between min-w-0 gap-4",
                        // Logo - use Dioxus Link for client-side navigation
                        Link {
                            to: Route::HomeScreen {},
                            class: "flex items-center gap-2 font-bold text-xl min-w-0",
                            span { class: "truncate", "{BRAND.app_name}" }
                        }
                        div { class: "flex items-center gap-3 ml-auto shrink-0",
                            a {
                                href: "{BRAND.repo_url}",
                                target: "_blank",
                                class: "icon-button",
                                div { class: "w-4 h-4",
                                    Icon { icon: LdGithub }
                                }
                            }
                            button {
                                onclick: toggle_dark_mode,
                                class: "icon-button",
                                aria_label: "Toggle theme",
                                if (*dark_theme.read()).0 {
                                    Icon { icon: LdSun, class: "w-5 h-5" }
                                } else {
                                    Icon { icon: LdMoon, class: "w-5 h-5" }
                                }
                            }

                            // User menu - use Dioxus Link for client-side navigation (only with consumer-auth feature)
                            { auth_ui }
                        }
                    }
                }
            }

            // Page content
            main { class: "flex-1",
                Outlet::<Route> {}
            }

            // Footer
            footer { class: "footer-container",
                div { class: "container mx-auto px-4 py-10 max-w-6xl",
                    div { class: "flex flex-col items-center gap-6 text-center",
                        // Navigation links
                        div { class: "flex flex-wrap items-center justify-center gap-4 text-sm font-mono",
                            Link {
                                to: Route::AboutScreen {},
                                class: "text-foreground/70 hover:text-foreground transition-colors",
                                "About"
                            }
                            span { class: "text-foreground/30", "·" }
                            Link {
                                to: Route::ContactScreen {},
                                class: "text-foreground/70 hover:text-foreground transition-colors",
                                "Contact"
                            }
                            span { class: "text-foreground/30", "·" }
                            Link {
                                to: Route::PrivacyPolicyScreen {},
                                class: "text-foreground/70 hover:text-foreground transition-colors",
                                "Privacy"
                            }
                            span { class: "text-foreground/30", "·" }
                            Link {
                                to: Route::TermsScreen {},
                                class: "text-foreground/70 hover:text-foreground transition-colors",
                                "Terms"
                            }
                        }

                        // Social icons
                        div { class: "flex items-center gap-3",
                            a {
                                href: "{BRAND.x_url}",
                                target: "_blank",
                                rel: "noopener",
                                class: "text-foreground/90 hover:text-foreground transition-colors",
                                aria_label: "X",
                                Icon { icon: SiX, class: "size-6" }
                            }
                            a {
                                href: "{BRAND.repo_url}",
                                target: "_blank",
                                rel: "noopener",
                                class: "text-foreground/90 hover:text-foreground transition-colors",
                                aria_label: "GitHub",
                                Icon { icon: LdGithub, class: "size-6" }
                            }
                        }

                        // Copyright
                        div { class: "text-sm font-mono",
                            "Copyright {BRAND.copyright_year} "
                            a {
                                href: "{BRAND.author_url}",
                                target: "_blank",
                                rel: "noopener",
                                class: "hover:underline",
                                "{BRAND.author}"
                            }
                            ". Built with "
                            a {
                                href: "{BRAND.dioxus_url}",
                                target: "_blank",
                                rel: "noopener",
                                class: "hover:underline",
                                "Dioxus"
                            }
                            " and "
                            a {
                                href: "{BRAND.rust_url}",
                                target: "_blank",
                                rel: "noopener",
                                class: "hover:underline",
                                "Rust"
                            }
                        }
                    }
                }
            }
        }
    }
}
