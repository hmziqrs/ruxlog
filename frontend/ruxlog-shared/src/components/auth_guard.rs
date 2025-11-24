use crate::store::auth::{use_auth, AuthUser};
use dioxus::prelude::*;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};
use std::sync::Arc;
use std::time::Duration;

const FADE_MS: u64 = 400;

pub type PermissionCheck = Arc<dyn Fn(Option<&AuthUser>) -> bool + Send + Sync>;
pub type RouteMatcherFn<R> = Arc<dyn Fn(&R, &R) -> bool + Send + Sync>;

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
                    // img {
                    //     class: "h-12 w-12 relative",
                    //     src: asset!("/assets/logo.png"),
                    //     alt: "Ruxlog",
                    // }
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
                        // img {
                        //     class: "h-24 w-24",
                        //     src: asset!("/assets/logo.png"),
                        //     alt: "Logo",
                        // }
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

#[derive(Props, Clone)]
pub struct AuthGuardContainerProps<R: Clone + PartialEq + 'static> {
    /// List of routes that don't require authentication
    pub open_routes: Vec<R>,
    /// Function to check if user has required permissions (optional)
    #[props(default)]
    pub permission_check: Option<PermissionCheck>,
    /// Custom route matcher function (optional, defaults to PartialEq)
    /// This allows matching routes with parameters (e.g., PostViewScreen { id: _ })
    #[props(default)]
    pub route_matcher: Option<RouteMatcherFn<R>>,
    /// Route to redirect to when not authenticated
    pub login_route: R,
    /// Route to redirect to when authenticated but on open routes
    pub authenticated_route: R,
    /// Custom loading messages (optional)
    #[props(default)]
    pub loading_messages: Option<LoadingMessages>,
}

impl<R: Clone + PartialEq + 'static> PartialEq for AuthGuardContainerProps<R> {
    fn eq(&self, other: &Self) -> bool {
        self.open_routes == other.open_routes
            && self.login_route == other.login_route
            && self.authenticated_route == other.authenticated_route
            && self.loading_messages == other.loading_messages
            // We can't compare the functions, so we just check if they exist
            && (self.permission_check.is_some() == other.permission_check.is_some())
            && (self.route_matcher.is_some() == other.route_matcher.is_some())
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LoadingMessages {
    pub init: Option<(String, String)>,
    pub login: Option<(String, String)>,
    pub logout: Option<(String, String)>,
    pub default: Option<(String, String)>,
}

#[component]
pub fn AuthGuardContainer<R: Clone + PartialEq + 'static>(
    props: AuthGuardContainerProps<R>,
) -> Element
where
    R: Clone + PartialEq + 'static + dioxus::prelude::Routable,
{
    let render_blocked = use_signal(|| true);
    let auth_store = crate::use_auth();
    let nav = use_navigator();
    let route: R = use_route();

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
    let open_routes = props.open_routes.clone();
    let permission_check = props.permission_check.clone();
    let route_matcher = props.route_matcher.clone();
    let login_route = props.login_route.clone();
    let authenticated_route = props.authenticated_route.clone();

    use_effect(use_reactive!(|(route_for_logic)| {
        let init_status = auth_store.init_status.read();
        if init_status.is_success() {
            let user = auth_store.user.read().clone();
            let is_open_route = if let Some(matcher) = &route_matcher {
                open_routes.iter().any(|r| matcher(r, &route_for_logic))
            } else {
                open_routes.iter().any(|r| r == &route_for_logic)
            };
            let nav = nav_for_logic.clone();
            let _route = route_for_logic.clone();

            let permission_check = permission_check.clone();
            let login_route = login_route.clone();
            let authenticated_route = authenticated_route.clone();
            let route_matcher = route_matcher.clone();

            spawn(async move {
                let is_logged_in = user.is_some();
                let has_permission = permission_check
                    .as_ref()
                    .map(|check| check(user.as_ref()))
                    .unwrap_or(true);
                let is_on_authenticated_route = if let Some(matcher) = &route_matcher {
                    matcher(&authenticated_route, &_route)
                } else {
                    authenticated_route == _route
                };

                // Check permissions first
                if is_logged_in && !has_permission {
                    auth_store.logout().await;
                    nav.push(login_route);
                    return;
                }

                if !is_logged_in && !is_open_route {
                    nav.push(login_route);
                    return;
                }

                // Redirect authenticated users from open/auth routes
                if is_logged_in && is_open_route {
                    if !is_on_authenticated_route {
                        nav.push(authenticated_route);
                    }
                    render_blocked_for_logic.set(false);
                    return;
                }

                render_blocked_for_logic.set(false);
            });
        }
    }));

    let init_status = auth_store.init_status.read();
    let login_status = auth_store.login_status.read();
    let logout_status = auth_store.logout_status.read();

    let default_messages = LoadingMessages::default();
    let messages = props.loading_messages.as_ref().unwrap_or(&default_messages);

    let (loader_title, loader_copy) = if init_status.is_loading() {
        messages.init.clone().unwrap_or_else(|| {
            (
                "Checking your session…".to_string(),
                "Hold tight while we verify your account and get things ready.".to_string(),
            )
        })
    } else if login_status.is_loading() {
        messages.login.clone().unwrap_or_else(|| {
            (
                "Signing you in…".to_string(),
                "Validating your credentials and preparing your workspace.".to_string(),
            )
        })
    } else if logout_status.is_loading() {
        messages.logout.clone().unwrap_or_else(|| {
            (
                "Signing you out…".to_string(),
                "Wrapping up and clearing your session securely.".to_string(),
            )
        })
    } else {
        messages.default.clone().unwrap_or_else(|| {
            (
                "Preparing…".to_string(),
                "Bringing everything online.".to_string(),
            )
        })
    };

    let show_overlay = use_memo(move || {
        let login = auth_store.login_status.read();
        let logout = auth_store.logout_status.read();
        login.is_loading() || logout.is_loading()
    });

    let (overlay_title, overlay_copy) = if login_status.is_loading() {
        messages.login.clone().unwrap_or_else(|| {
            (
                "Signing you in…".to_string(),
                "Validating your credentials and preparing your workspace.".to_string(),
            )
        })
    } else if logout_status.is_loading() {
        messages.logout.clone().unwrap_or_else(|| {
            (
                "Signing you out…".to_string(),
                "Wrapping up and clearing your session securely.".to_string(),
            )
        })
    } else {
        ("".to_string(), "".to_string())
    };

    if init_status.is_failed() {
        return rsx! {
            AuthGuardError {
                on_retry: move |_| {
                    spawn(async move {
                        auth_store.init().await;
                    });
                },
            }
        };
    }

    rsx! {
        AuthGuardLoader { title: loader_title, copy: loader_copy, show: render_blocked }
        if !render_blocked() {
            Outlet::<R> {}
            AuthGuardLoader {
                title: overlay_title,
                copy: overlay_copy,
                overlay: true,
                show: show_overlay,
            }
        }
    }
}
