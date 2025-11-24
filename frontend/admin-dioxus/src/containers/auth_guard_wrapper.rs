//! Admin Auth Guard
//!
//! Admin-specific authentication guard that:
//! - Requires authentication for ALL routes except LoginScreen
//! - Requires admin role for access
//! - Redirects non-admins to login after logout

use crate::router::Route;
use dioxus::prelude::*;
use ruxlog_shared::{use_auth, AuthGuardError, AuthGuardLoader};

/// Routes that don't require authentication (only login screen for admin)
const OPEN_ROUTES: &[Route] = &[Route::LoginScreen {}];

#[component]
pub fn AuthGuardContainer() -> Element {
    let render_blocked = use_signal(|| true);
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

    // Handle auth logic on route changes
    let nav_for_logic = nav.clone();
    let mut render_blocked_for_logic = render_blocked.clone();
    let route_for_logic = route.clone();

    use_effect(use_reactive!(|(route_for_logic)| {
        let init_status = auth_store.init_status.read();
        if init_status.is_success() {
            let user = auth_store.user.read().clone();
            let is_open_route = OPEN_ROUTES.iter().any(|r| r == &route_for_logic);
            let is_logged_in = user.is_some();
            let is_admin = user.as_ref().map(|u| u.is_admin()).unwrap_or(false);
            let nav = nav_for_logic.clone();
            let route = route_for_logic.clone();

            spawn(async move {
                // Non-admin users get logged out and redirected to login
                if is_logged_in && !is_admin {
                    auth_store.logout().await;
                    if !matches!(route, Route::LoginScreen { .. }) {
                        nav.replace(Route::LoginScreen {});
                    }
                    return;
                }

                // Logged-in admins on login page get redirected to home
                if is_logged_in && is_open_route {
                    render_blocked_for_logic.set(false);
                    nav.replace(Route::HomeScreen {});
                    return;
                }

                // Non-logged-in users on protected routes get redirected to login
                if !is_logged_in && !is_open_route {
                    nav.replace(Route::LoginScreen {});
                    return;
                }

                render_blocked_for_logic.set(false);
            });
        }
    }));

    // Read status for UI
    let init_status = auth_store.init_status.read();
    let login_status = auth_store.login_status.read();
    let logout_status = auth_store.logout_status.read();

    // Determine loader messages
    let (loader_title, loader_copy) = if init_status.is_loading() {
        (
            "Checking your workspace…".to_string(),
            "Hold tight while we verify your session and load the dashboard.".to_string(),
        )
    } else if login_status.is_loading() {
        (
            "Signing you in…".to_string(),
            "Validating your credentials and preparing your workspace.".to_string(),
        )
    } else if logout_status.is_loading() {
        (
            "Signing you out…".to_string(),
            "Wrapping up and clearing your session securely.".to_string(),
        )
    } else {
        (
            "Preparing dashboard…".to_string(),
            "Bringing everything online for your next task.".to_string(),
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
            "Validating your credentials and preparing your workspace.".to_string(),
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
            show: render_blocked,
        }
        if !render_blocked() {
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
