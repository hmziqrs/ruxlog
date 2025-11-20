use dioxus::prelude::*;
use crate::router::Route;
use ruxlog_shared::{AuthGuardContainer as SharedAuthGuardContainer, LoadingMessages, PermissionCheck};
use std::sync::Arc;

#[component]
pub fn AuthGuardContainer() -> Element {
    rsx! {
        SharedAuthGuardContainer::<Route> {
            open_routes: vec![Route::LoginScreen {}],
            permission_check: Some(Arc::new(|user: Option<&ruxlog_shared::AuthUser>| {
                user.map(|u| u.is_admin()).unwrap_or(false)
            }) as PermissionCheck),
            login_route: Route::LoginScreen {},
            authenticated_route: Route::HomeScreen {},
            loading_messages: Some(LoadingMessages {
                init: Some((
                    "Checking your workspace…".to_string(),
                    "Hold tight while we verify your session and load the dashboard.".to_string(),
                )),
                login: Some((
                    "Signing you in…".to_string(),
                    "Validating your credentials and preparing your workspace.".to_string(),
                )),
                logout: Some((
                    "Signing you out…".to_string(),
                    "Wrapping up and clearing your session securely.".to_string(),
                )),
                default: Some((
                    "Preparing dashboard…".to_string(),
                    "Bringing everything online for your next task.".to_string(),
                )),
            }),
        }
    }
}