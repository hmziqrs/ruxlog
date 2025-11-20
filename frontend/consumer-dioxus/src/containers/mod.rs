use dioxus::prelude::*;
// use dioxus_router::prelude::Outlet;

use crate::router::Route;

#[component]
pub fn NavBarContainer() -> Element {
    rsx! {
        div {
            "NavBar"
            Outlet::<Route> {}
        }
    }
}

#[component]
pub fn AuthGuardContainer() -> Element {
    rsx! {
        div {
            "Auth Guard"
            Outlet::<Route> {}
        }
    }
}
