use dioxus::prelude::*;
use crate::router::Route;
use ruxlog_shared::use_auth;
use crate::config::DarkMode;
use crate::utils::persist;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdMenu, LdMoon, LdSun, LdUser, LdLogIn};
use hmziq_dioxus_free_icons::Icon;

pub mod auth_guard_wrapper;
pub use auth_guard_wrapper::*;

#[component]
pub fn NavBarContainer() -> Element {
    let auth_store = use_auth();
    let user = auth_store.user.read();
    let mut dark_theme = use_context_provider(|| Signal::new(DarkMode(true)));
    let mut mobile_menu_open = use_signal(|| false);

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

                        // Desktop navigation
                        div { class: "hidden md:flex items-center gap-6",
                            a {
                                href: "/",
                                class: "text-sm font-medium text-foreground/80 hover:text-foreground transition-colors",
                                "Home"
                            }
                            a {
                                href: "/categories",
                                class: "text-sm font-medium text-foreground/80 hover:text-foreground transition-colors",
                                "Categories"
                            }
                            a {
                                href: "/about",
                                class: "text-sm font-medium text-foreground/80 hover:text-foreground transition-colors",
                                "About"
                            }
                        }

                        // Actions
                        div { class: "flex items-center gap-3",
                            // Theme toggle
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
                                    class: "hidden md:flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold text-sm",
                                        "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                                    }
                                    span { class: "text-sm font-medium", "{user.name}" }
                                }
                            } else {
                                a {
                                    href: "/login",
                                    class: "hidden md:flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors text-sm font-medium",
                                    Icon { icon: LdLogIn, class: "w-4 h-4" }
                                    "Sign In"
                                }
                            }

                            // Mobile menu button
                            button {
                                onclick: move |_| mobile_menu_open.toggle(),
                                class: "md:hidden p-2 rounded-lg hover:bg-muted/50 transition-colors",
                                Icon { icon: LdMenu, class: "w-5 h-5" }
                            }
                        }
                    }

                    // Mobile menu
                    if mobile_menu_open() {
                        div { class: "md:hidden py-4 border-t border-border/60",
                            div { class: "flex flex-col gap-3",
                                a {
                                    href: "/",
                                    class: "px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    "Home"
                                }
                                a {
                                    href: "/categories",
                                    class: "px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    "Categories"
                                }
                                a {
                                    href: "/about",
                                    class: "px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                    "About"
                                }
                                
                                div { class: "border-t border-border/60 my-2" }
                                
                                if let Some(_user) = &*user {
                                    a {
                                        href: "/profile",
                                        class: "flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted/50 transition-colors",
                                        Icon { icon: LdUser, class: "w-5 h-5" }
                                        "Profile"
                                    }
                                } else {
                                    a {
                                        href: "/login",
                                        class: "flex items-center gap-2 px-3 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors",
                                        Icon { icon: LdLogIn, class: "w-4 h-4" }
                                        "Sign In"
                                    }
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
                div { class: "container mx-auto px-4 py-12",
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-8",
                        // Brand
                        div {
                            h3 { class: "font-bold text-lg mb-4", "Ruxlog" }
                            p { class: "text-sm text-muted-foreground", "A modern blogging platform for sharing ideas and stories." }
                        }

                        // Links
                        div {
                            h4 { class: "font-semibold mb-4", "Explore" }
                            div { class: "flex flex-col gap-2 text-sm",
                                a { href: "/", class: "text-muted-foreground hover:text-foreground transition-colors", "Home" }
                                a { href: "/categories", class: "text-muted-foreground hover:text-foreground transition-colors", "Categories" }
                                a { href: "/tags", class: "text-muted-foreground hover:text-foreground transition-colors", "Tags" }
                            }
                        }

                        // Company
                        div {
                            h4 { class: "font-semibold mb-4", "Company" }
                            div { class: "flex flex-col gap-2 text-sm",
                                a { href: "/about", class: "text-muted-foreground hover:text-foreground transition-colors", "About" }
                                a { href: "/contact", class: "text-muted-foreground hover:text-foreground transition-colors", "Contact" }
                                a { href: "/privacy", class: "text-muted-foreground hover:text-foreground transition-colors", "Privacy" }
                            }
                        }

                        // Social
                        div {
                            h4 { class: "font-semibold mb-4", "Connect" }
                            div { class: "flex flex-col gap-2 text-sm",
                                a { href: "#", class: "text-muted-foreground hover:text-foreground transition-colors", "Twitter" }
                                a { href: "#", class: "text-muted-foreground hover:text-foreground transition-colors", "GitHub" }
                                a { href: "#", class: "text-muted-foreground hover:text-foreground transition-colors", "LinkedIn" }
                            }
                        }
                    }

                    div { class: "mt-8 pt-8 border-t border-border/60 text-center text-sm text-muted-foreground",
                        "© 2024 Ruxlog. All rights reserved."
                    }
                }
            }
        }
    }
}

