use dioxus::prelude::*;

use crate::{
    router::{Route, OPEN_ROUTES},
    store::use_auth,
};

use super::{AuthGuardError, AuthGuardLoader};

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
        } else if init_status_hook.is_success() {
            //
        }
    });

    // For the second effect, we need to re-read init_status and user inside the closure
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
                if is_logged_in && !is_admin {
                    auth_store.logout().await;
                    if !matches!(route, Route::LoginScreen { .. }) {
                        nav.replace(Route::LoginScreen {});
                    }
                    return;
                }

                if is_logged_in && is_open_route {
                    render_blocked_for_logic.set(false);
                    nav.replace(Route::HomeScreen {});
                    return;
                }
                if !is_logged_in && !is_open_route {
                    nav.replace(Route::LoginScreen {});
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
            "Checking your workspace…",
            "Hold tight while we verify your session and load the dashboard.",
        )
    } else if login_status.is_loading() {
        (
            "Signing you in…",
            "Validating your credentials and preparing your workspace.",
        )
    } else if logout_status.is_loading() {
        (
            "Signing you out…",
            "Wrapping up and clearing your session securely.",
        )
    } else {
        (
            "Preparing dashboard…",
            "Bringing everything online for your next task.",
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
            "Validating your credentials and preparing your workspace.",
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
