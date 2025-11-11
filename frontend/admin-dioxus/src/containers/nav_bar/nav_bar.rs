use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdBell, LdMenu, LdMoon, LdSun};
use hmziq_dioxus_free_icons::Icon;

use crate::components::{Sidebar, UserAvatar};
use crate::config::DarkMode;
use crate::{router::Route, store::use_auth, utils::persist};

#[component]
pub fn NavBarContainer() -> Element {
    let auth_store = use_auth();
    let auth_user = auth_store.user.read();
    let mut sidebar_open = use_signal(|| false);
    let mut dark_theme = use_context_provider(|| Signal::new(DarkMode(true)));

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
        // Update DOM immediately and persist preference via bevy_pkv and localStorage (handled in persist)
        spawn(async move {
            _ = document::eval("document.documentElement.classList.toggle('dark');").await;
        });
        persist::set_theme(if is_dark { "dark" } else { "light" });
    };

    let toggle_sidebar = move |_: MouseEvent| {
        sidebar_open.toggle();
    };

    if auth_user.is_none() {
        return rsx! {
            Outlet::<Route> {}
        };
    }

    rsx! {
        Sidebar { expanded: sidebar_open, toggle: move |_| sidebar_open.toggle() }

        header { class: "sticky top-0 z-20 border-b border-border/60 backdrop-blur-xl transition-colors duration-300",
            div { class: "flex h-16 items-center justify-between px-4",
                div { class: "flex items-center",
                    button {
                        class: "rounded-md p-2 text-muted-foreground transition-colors duration-200 hover:bg-muted/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background sm:hidden",
                        onclick: move |_| sidebar_open.set(true),
                        div { class: "w-4 h-4",
                            Icon { icon: LdMenu }
                        }
                    }
                    button {
                        class: "hidden rounded-md p-2 text-muted-foreground transition-colors duration-200 hover:bg-muted/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background sm:flex",
                        onclick: toggle_sidebar,
                        div { class: "w-4 h-4",
                            Icon { icon: LdMenu }
                        }
                    }
                }

                div { class: "flex items-center space-x-4",
                    button { class: "rounded-full p-1 text-muted-foreground transition-colors duration-200 hover:bg-muted/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                        div { class: "w-4 h-4",
                            Icon { icon: LdBell }
                        }
                    }
                    button {
                        onclick: toggle_dark_mode,
                        class: "rounded-full p-1 text-muted-foreground transition-colors duration-200 hover:bg-muted/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                        div { class: "w-4 h-4",
                            if (*dark_theme.read()).0 {
                                Icon { icon: LdSun }
                            } else {
                                Icon { icon: LdMoon }
                            }
                        }
                    }
                    div { class: "relative ml-3",
                        if let Some(user) = auth_user.as_ref() {
                            UserAvatar {
                                name: user.name.clone(),
                                avatar: user.avatar.clone(),
                            }
                        }
                    }
                }
            }
        }

        Outlet::<Route> {}
    }
}
