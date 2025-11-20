use dioxus::prelude::*;
use std::time::Duration;
use crate::router::{Route, OPEN_ROUTES};
use ruxlog_shared::use_auth;
use crate::config::DarkMode;
use crate::utils::persist;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdMenu, LdMoon, LdSun, LdUser, LdLogIn};
use hmziq_dioxus_free_icons::Icon;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use oxui::shadcn::button::{Button, ButtonVariant};

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
                                
                                if let Some(user) = &*user {
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

#[component]
pub fn AuthGuardContainer() -> Element {
    let render_blocked = use_signal(|| true);

    let auth_store = use_auth();
    let nav = use_navigator();
    let route: Route = use_route();

    let init_status = auth_store.init_status.read();
    let init_status_hook = init_status.clone();

    use_effect(move || {
        if init_status_hook.is_init() {
            spawn(async move {
                auth_store.init().await;
            });
        }
    });

    let nav_for_logic = nav.clone();
    let mut render_blocked_for_logic = render_blocked.clone();
    let route_for_logic = route.clone();

    use_effect(use_reactive!(|(route_for_logic)| {
        let init_status = auth_store.init_status.read();
        if init_status.is_success() {
            let user = auth_store.user.read().clone();
            let is_open_route = OPEN_ROUTES.iter().any(|r| r == &route_for_logic);
            let nav = nav_for_logic.clone();
            let route = route_for_logic.clone();

            spawn(async move {
                let is_logged_in = user.is_some();

                if !is_logged_in && !is_open_route {
                    nav.replace(Route::LoginScreen {});
                    return;
                }

                // Signed-in visitors shouldn't see the auth pages
                if is_logged_in && matches!(route, Route::LoginScreen {} | Route::RegisterScreen {}) {
                    nav.replace(Route::HomeScreen {});
                    return;
                }

                render_blocked_for_logic.set(false);
            });
        }
    }));

    let init_status = auth_store.init_status.read();
    let login_status = auth_store.login_status.read();
    let logout_status = auth_store.logout_status.read();

    let (loader_title, loader_copy) = if init_status.is_loading() {
        (
            "Checking your session…",
            "Hold tight while we verify your account and get things ready.",
        )
    } else if login_status.is_loading() {
        (
            "Signing you in…",
            "Validating your credentials and preparing your feed.",
        )
    } else if logout_status.is_loading() {
        (
            "Signing you out…",
            "Wrapping up and clearing your session securely.",
        )
    } else {
        (
            "Preparing your feed…",
            "Bringing everything online for your next read.",
        )
    };

    let show_overlay = use_memo(move || {
        let login = auth_store.login_status.read();
        let logout = auth_store.logout_status.read();
        login.is_loading() || logout.is_loading()
    });

    let (overlay_title, overlay_copy) = if login_status.is_loading() {
        (
            "Signing you in…",
            "Validating your credentials and preparing your feed.",
        )
    } else if logout_status.is_loading() {
        (
            "Signing you out…",
            "Wrapping up and clearing your session securely.",
        )
    } else {
        ("", "")
    };

    if init_status.is_failed() {
        return rsx! {
            AuthGuardError {
                on_retry: move |_| {
                    spawn(async move {
                        auth_store.init().await;
                    });
                }
            }
        };
    }

    rsx! {
        AuthGuardLoader {
            title: loader_title.to_string(),
            copy: loader_copy.to_string(),
            show: render_blocked,
        }
        if !render_blocked() {
            Outlet::<Route> {}
            AuthGuardLoader {
                title: overlay_title.to_string(),
                copy: overlay_copy.to_string(),
                overlay: true,
                show: show_overlay,
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct AuthGuardLoaderProps {
    title: String,
    copy: String,
    #[props(default = false)]
    overlay: bool,
    pub show: ReadSignal<bool>,
}

#[component]
pub fn AuthGuardLoader(props: AuthGuardLoaderProps) -> Element {
    let container_class = if props.overlay {
        "fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center px-4"
    } else {
        "min-h-screen bg-background flex items-center justify-center px-4"
    };

    const FADE_MS: u64 = 400;

    let mut should_render = use_signal(|| false);
    let show_sig = props.show.clone();
    let show_for_render = show_sig.clone();

    use_effect(move || {
        let is_show = show_sig();
        if is_show && !should_render() {
            should_render.set(true);
        } else if !is_show && should_render() {
            spawn(async move {
                dioxus_time::sleep(Duration::from_millis(FADE_MS)).await;
                should_render.set(false);
            });
        }
    });

    if !should_render() {
        return rsx! { Fragment {} };
    }

    let opacity = if show_for_render() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div { class: "{container_class} duration-400 ease-in-out {opacity}",
            div { class: "w-full max-w-sm text-center space-y-6",
                div { class: "relative mx-auto h-24 w-24 flex items-center justify-center",
                    div { class: "absolute inset-0 rounded-full border-4 border-primary/20 border-t-primary animate-spin" }
                    img {
                        class: "h-12 w-12 relative",
                        src: asset!("/assets/logo.png"),
                        alt: "Ruxlog",
                    }
                }
                div { class: "space-y-2",
                    p { class: "text-lg font-semibold text-foreground", "{props.title}" }
                    p { class: "text-sm text-muted-foreground", "{props.copy}" }
                }
            }
        }
    }
}

#[component]
pub fn AuthGuardError(on_retry: EventHandler<()>) -> Element {
    let auth_store = use_auth();
    let init_status = auth_store.init_status.read();

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-background p-4",
            div { class: "max-w-md w-full",
                div { class: "rounded-xl border border-border/60 bg-background p-8 shadow-lg space-y-6",
                    div { class: "flex justify-center mb-2",
                        img {
                            class: "h-24 w-24",
                            src: asset!("/assets/logo.png"),
                            alt: "Logo",
                        }
                    }
                    ErrorDetails {
                        error: init_status.error.clone(),
                        variant: ErrorDetailsVariant::Minimum,
                        class: Some("w-full".to_string()),
                    }
                    div { class: "flex justify-center pt-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_retry.call(()),
                            "Try Again"
                        }
                    }
                }
            }
        }
    }
}
