use dioxus::prelude::*;
use crate::router::Route;
use ruxlog_shared::use_auth;
use crate::config::DarkMode;
use crate::utils::persist;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdMoon, LdSun, LdLogIn, LdGithub, LdTwitter, LdLinkedin};
use hmziq_dioxus_free_icons::Icon;

pub mod auth_guard_wrapper;
pub use auth_guard_wrapper::*;

#[component]
pub fn NavBarContainer() -> Element {
    let auth_store = use_auth();
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

    rsx! {
        div { class: "min-h-screen bg-background",
            // Navbar
            nav { class: "sticky top-0 z-50 border-b border-border/60 backdrop-blur-xl bg-background/80",
                div { class: "container mx-auto px-4",
                    div { class: "flex h-16 items-center justify-between",
                        // Logo
                        a {
                            href: "/",
                            class: "flex items-center gap-2 font-bold text-xl",
                            span { class: "text-primary", "Ruxlog" }
                        }
                        div { class: "flex items-center gap-3 ml-auto",
                            a {
                                href: "https://github.com/hmziqrs/ruxlog",
                                target: "_blank",
                                class: "p-2 rounded-lg hover:bg-muted/50 transition-colors",
                                div { class: "w-4 h-4",
                                    Icon { icon: LdGithub }
                                }
                            }
                            button {
                                onclick: toggle_dark_mode,
                                class: "p-2 rounded-lg hover:bg-muted/50 transition-colors",
                                aria_label: "Toggle theme",
                                if (*dark_theme.read()).0 {
                                    Icon { icon: LdSun, class: "w-5 h-5" }
                                } else {
                                    Icon { icon: LdMoon, class: "w-5 h-5" }
                                }
                            }

                            // User menu
                            if let Some(user) = &*user {
                                a {
                                    href: "/profile",
                                    class: "flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold text-sm",
                                        "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                                    }
                                    span { class: "hidden md:block text-sm font-medium", "{user.name}" }
                                }
                            } else {
                                a {
                                    href: "/login",
                                    class: "p-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    aria_label: "Sign In",
                                    Icon { icon: LdLogIn, class: "w-5 h-5" }
                                }
                            }
                        }
                    }
                }
            }

            // Page content
            Outlet::<Route> {}

            // Footer
            footer { class: "border-t border-border/60 mt-auto",
                div { class: "container mx-auto px-4 py-8",
                    div { class: "flex flex-col items-center gap-6",
                        // Navigation links
                        div { class: "flex items-center gap-6",
                            a {
                                href: "/about",
                                class: "text-sm text-muted-foreground hover:text-foreground transition-colors",
                                "About"
                            }
                            a {
                                href: "/contact",
                                class: "text-sm text-muted-foreground hover:text-foreground transition-colors",
                                "Contact"
                            }
                        }

                        // Social icons
                        div { class: "flex items-center gap-4",
                            a {
                                href: "https://twitter.com",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "p-2 rounded-lg hover:bg-muted/50 transition-colors text-muted-foreground hover:text-foreground",
                                aria_label: "Twitter",
                                Icon { icon: LdTwitter, class: "w-5 h-5" }
                            }
                            a {
                                href: "https://github.com",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "p-2 rounded-lg hover:bg-muted/50 transition-colors text-muted-foreground hover:text-foreground",
                                aria_label: "GitHub",
                                Icon { icon: LdGithub, class: "w-5 h-5" }
                            }
                            a {
                                href: "https://linkedin.com",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "p-2 rounded-lg hover:bg-muted/50 transition-colors text-muted-foreground hover:text-foreground",
                                aria_label: "LinkedIn",
                                Icon { icon: LdLinkedin, class: "w-5 h-5" }
                            }
                        }

                        // Built with message
                        div { class: "text-center text-sm text-muted-foreground",
                            "Built from scratch with "
                            a {
                                href: "https://dioxuslabs.com",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-primary hover:underline",
                                "Dioxus"
                            }
                            " by "
                            a {
                                href: "https://hmziq.rs",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-primary hover:underline",
                                "hmziqrs"
                            }
                        }

                        // Copyright
                        div { class: "text-center text-sm text-muted-foreground",
                            "© 2024 Ruxlog. All rights reserved."
                        }
                    }
                }
            }
        }
    }
}

