//! Consumer Auth Guard
//!
//! Consumer-specific authentication guard for a public site that:
//! - Allows access to ALL routes (public site)
//! - Only redirects logged-in users away from auth pages (login/register)
//! - Does NOT block rendering during route changes
//! - Shows loading only during auth initialization

use crate::router::Route;
use dioxus::prelude::*;
use ruxlog_shared::{use_auth, AuthGuardError, AuthGuardLoader};

/// Auth routes - routes only for unauthenticated users (login, register)
const AUTH_ROUTES: &[Route] = &[Route::LoginScreen {}, Route::RegisterScreen {}];

#[component]
pub fn AuthGuardContainer() -> Element {
    // Only block on initial auth check, not on route changes
    let init_blocked = use_signal(|| true);
    let auth_store = use_auth();
    let nav = use_navigator();
    let route: Route = use_route();

    // Initialize auth on mount
    let init_status = auth_store.init_status.read();
    let init_status_hook = init_status.clone();

    use_effect(move || {
        if init_status_hook.is_init() {
            spawn(async move {
                auth_store.init().await;
            });
        }
    });

    // Handle auth redirects (only for auth pages like login/register)
    let nav_for_logic = nav.clone();
    let mut init_blocked_for_logic = init_blocked.clone();
    let route_for_logic = route.clone();

    use_effect(use_reactive!(|(route_for_logic)| {
        let init_status = auth_store.init_status.read();
        if init_status.is_success() {
            let user = auth_store.user.read().clone();
            let is_auth_route = AUTH_ROUTES.iter().any(|r| r == &route_for_logic);
            let is_logged_in = user.is_some();
            let nav = nav_for_logic.clone();

            // Only redirect: logged-in users shouldn't see login/register pages
            if is_logged_in && is_auth_route {
                nav.replace(Route::HomeScreen {});
            }

            // Always unblock - this is a public site
            init_blocked_for_logic.set(false);
        }
    }));

    // Read status for UI
    let init_status = auth_store.init_status.read();
    let login_status = auth_store.login_status.read();
    let logout_status = auth_store.logout_status.read();

    // Determine loader messages (only shown during init)
    let (loader_title, loader_copy) = if init_status.is_loading() {
        (
            "Checking your session…".to_string(),
            "Hold tight while we verify your account and get things ready.".to_string(),
        )
    } else {
        (
            "Preparing your feed…".to_string(),
            "Bringing everything online for your next read.".to_string(),
        )
    };

    // Overlay for login/logout operations
    let show_overlay = use_memo(move || {
        let login = auth_store.login_status.read();
        let logout = auth_store.logout_status.read();
        login.is_loading() || logout.is_loading()
    });

    let (overlay_title, overlay_copy) = if login_status.is_loading() {
        (
            "Signing you in…".to_string(),
            "Validating your credentials and preparing your feed.".to_string(),
        )
    } else if logout_status.is_loading() {
        (
            "Signing you out…".to_string(),
            "Wrapping up and clearing your session securely.".to_string(),
        )
    } else {
        ("".to_string(), "".to_string())
    };

    // Show error screen if init failed
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
            title: loader_title,
            copy: loader_copy,
            show: init_blocked,
        }
        if !init_blocked() {
            Outlet::<Route> {}
            AuthGuardLoader {
                title: overlay_title,
                copy: overlay_copy,
                overlay: true,
                show: show_overlay,
            }
        }
    }
}
