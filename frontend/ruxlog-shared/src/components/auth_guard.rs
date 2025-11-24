//! Shared Auth Guard UI Components
//!
//! This module provides reusable UI components for auth guards:
//! - `AuthGuardLoader` - Loading spinner with customizable messages
//! - `AuthGuardError` - Error display with retry functionality  
//! - `LoadingMessages` - Configuration for loading message text
//!
//! The actual auth logic should be implemented separately in each app
//! since admin and consumer have fundamentally different auth models:
//! - Admin: All routes require auth + admin role (except login)
//! - Consumer: Public site with optional auth, most routes accessible

use crate::store::auth::use_auth;
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use std::time::Duration;

const FADE_MS: u64 = 400;

/// Configuration for loading messages shown during auth states
#[derive(Default, Clone, PartialEq)]
pub struct LoadingMessages {
    pub init: Option<(String, String)>,
    pub login: Option<(String, String)>,
    pub logout: Option<(String, String)>,
    pub default: Option<(String, String)>,
}

#[derive(Props, PartialEq, Clone)]
pub struct AuthGuardLoaderProps {
    pub title: String,
    pub copy: String,
    #[props(default = false)]
    pub overlay: bool,
    pub show: ReadSignal<bool>,
}

/// Loading spinner component for auth guard
/// 
/// Shows a full-screen or overlay loading state with customizable messages.
/// Handles fade in/out animations automatically.
#[component]
pub fn AuthGuardLoader(props: AuthGuardLoaderProps) -> Element {
    let mut should_render = use_signal(|| false);
    let visible_sig = props.show.clone();

    use_effect(move || {
        let is_visible = visible_sig();
        if is_visible && !should_render() {
            // Show immediately
            should_render.set(true);
        } else if !is_visible && should_render() {
            // Delay hide to allow fade out
            spawn(async move {
                dioxus_time::sleep(Duration::from_millis(FADE_MS)).await;
                should_render.set(false);
            });
        }
    });

    if !should_render() {
        return rsx! {
            Fragment {}
        };
    }

    let container_class = if props.overlay {
        "fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center px-4"
    } else {
        "min-h-screen bg-background flex items-center justify-center px-4"
    };

    let opacity = if *props.show.read() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div { class: "{container_class} duration-400 ease-in-out {opacity}",
            div { class: "w-full max-w-sm text-center space-y-6",
                div { class: "relative mx-auto h-24 w-24 flex items-center justify-center",
                    div { class: "absolute inset-0 rounded-full border-4 border-primary/20 border-t-primary animate-spin" }
                    div { class: "h-12 w-12 relative flex items-center justify-center bg-primary/10 rounded-lg",
                        span { class: "text-primary font-bold text-lg", "R" }
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

/// Error display component for auth guard
/// 
/// Shows when auth initialization fails with a retry button.
#[component]
pub fn AuthGuardError(on_retry: EventHandler<()>) -> Element {
    let auth_store = use_auth();
    let init_status = auth_store.init_status.read();
    let mut is_visible = use_signal(|| false);

    // Entrance animation with next tick delay
    use_effect(move || {
        spawn(async move {
            dioxus_time::sleep(Duration::from_millis(16)).await;
            is_visible.set(true);
        });
    });

    let opacity = if is_visible() {
        "opacity-100"
    } else {
        "opacity-0"
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-background p-4 duration-400 ease-in-out {opacity}",
            div { class: "max-w-md w-full",
                div { class: "rounded-xl border border-border/60 bg-background p-8 shadow-lg space-y-6",
                    div { class: "flex justify-center mb-2",
                        div { class: "h-24 w-24 flex items-center justify-center bg-primary/10 rounded-lg",
                            span { class: "text-primary font-bold text-2xl", "R" }
                        }
                    }
                    div { class: "text-center space-y-3",
                        h3 { class: "text-lg font-semibold text-foreground", "Authentication Error" }
                        ErrorDetails {
                            error: init_status.error.clone(),
                            variant: ErrorDetailsVariant::Minimum,
                            class: "w-full",
                        }
                    }
                    div { class: "flex justify-center pt-2",
                        button {
                            onclick: move |_| on_retry.call(()),
                            class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium text-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
                            "Try Again"
                        }
                    }
                }
            }
        }
    }
}
