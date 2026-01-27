use crate::config::DarkMode;
use crate::router::Route;
use crate::utils::persist;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdGithub, LdLinkedin, LdMoon, LdSun, LdTwitter};
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
        div { class: "min-h-screen bg-background",
            // Navbar
            nav { class: "navbar-container",
                div { class: "container mx-auto px-4",
                    div { class: "flex h-16 items-center justify-between",
                        // Logo - use Dioxus Link for client-side navigation
                        Link {
                            to: Route::HomeScreen {},
                            class: "flex items-center gap-2 font-bold text-xl",
                            span { "Ruxlog" }
                        }
                        div { class: "flex items-center gap-3 ml-auto",
                            a {
                                href: "https://github.com/hmziqrs/ruxlog",
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
            Outlet::<Route> {}

            // Footer
            footer { class: "footer-container",
                div { class: "container mx-auto px-4 py-8 md:py-12",
                    div { class: "flex flex-col gap-8 md:flex-row md:items-start md:justify-between",
                        div { class: "flex flex-col items-center gap-6 md:items-end md:order-2",
                            // Navigation link groups
                            div { class: "flex flex-col gap-4 md:flex-row md:gap-8",
                                // Discover group
                                div { class: "flex flex-col items-center gap-2 md:items-start",
                                    Link {
                                        to: Route::TagsScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Tags"
                                    }
                                    Link {
                                        to: Route::CategoriesScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Categories"
                                    }
                                }

                                // Company group
                                div { class: "flex flex-col items-center gap-2 md:items-start",
                                    Link {
                                        to: Route::AboutScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "About"
                                    }
                                    Link {
                                        to: Route::ContactScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Contact"
                                    }
                                    Link {
                                        to: Route::AdvertiseScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Advertise"
                                    }
                                }

                                // Legal group
                                div { class: "flex flex-col items-center gap-2 md:items-start",
                                    Link {
                                        to: Route::PrivacyPolicyScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Privacy Policy"
                                    }
                                    Link {
                                        to: Route::TermsScreen {},
                                        class: "text-sm text-foreground/80 hover:text-foreground hover:underline transition-colors",
                                        "Terms"
                                    }
                                }
                            }

                            // Social icons (external links - keep as <a> tags)
                            div { class: "flex items-center gap-2 pt-2",
                                a {
                                    href: "https://twitter.com",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "icon-button opacity-70 hover:opacity-100",
                                    aria_label: "Twitter",
                                    Icon { icon: LdTwitter, class: "w-5 h-5" }
                                }
                                a {
                                    href: "https://github.com",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "icon-button opacity-70 hover:opacity-100",
                                    aria_label: "GitHub",
                                    Icon { icon: LdGithub, class: "w-5 h-5" }
                                }
                                a {
                                    href: "https://linkedin.com",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "icon-button opacity-70 hover:opacity-100",
                                    aria_label: "LinkedIn",
                                    Icon { icon: LdLinkedin, class: "w-5 h-5" }
                                }
                            }
                        }

                        div { class: "flex flex-col items-center gap-3 text-center md:items-start md:text-left md:order-1",
                            // Built with message (external links - keep as <a> tags)
                            div { class: "text-sm text-muted-foreground",
                                "Built from scratch with "
                                a {
                                    href: "https://dioxuslabs.com",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "hover:underline",
                                    "Dioxus"
                                }
                                " by "
                                a {
                                    href: "https://hmziq.rs",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "hover:underline",
                                    "hmziqrs"
                                }
                            }

                            // Copyright
                            div { class: "text-sm text-muted-foreground",
                                "© 2024 Ruxlog. All rights reserved."
                            }
                        }
                    }
                }
            }
        }
    }
}
