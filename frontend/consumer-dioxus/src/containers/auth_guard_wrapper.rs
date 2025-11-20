use dioxus::prelude::*;
use crate::router::{Route, OPEN_ROUTES};
use ruxlog_shared::{AuthGuardContainer as SharedAuthGuardContainer, LoadingMessages, RouteMatcherFn};
use std::sync::Arc;

#[component]
pub fn AuthGuardContainer() -> Element {
    rsx! {
        SharedAuthGuardContainer::<Route> {
            open_routes: OPEN_ROUTES.clone(),
            permission_check: None, // No admin permission check for consumer
            route_matcher: Some(Arc::new(|pattern: &Route, actual: &Route| {
                match (pattern, actual) {
                    // Match PostViewScreen regardless of id parameter
                    (Route::PostViewScreen { .. }, Route::PostViewScreen { .. }) => true,
                    // For all other routes, use exact equality
                    _ => pattern == actual,
                }
            }) as RouteMatcherFn<Route>),
            login_route: Route::LoginScreen {},
            authenticated_route: Route::HomeScreen {},
            loading_messages: Some(LoadingMessages {
                init: Some((
                    "Checking your session…".to_string(),
                    "Hold tight while we verify your account and get things ready.".to_string(),
                )),
                login: Some((
                    "Signing you in…".to_string(),
                    "Validating your credentials and preparing your feed.".to_string(),
                )),
                logout: Some((
                    "Signing you out…".to_string(),
                    "Wrapping up and clearing your session securely.".to_string(),
                )),
                default: Some((
                    "Preparing your feed…".to_string(),
                    "Bringing everything online for your next read.".to_string(),
                )),
            }),
        }
    }
}
