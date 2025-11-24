use dioxus::prelude::*;
use crate::containers::{AuthGuardContainer, NavBarContainer};
use crate::screens::{
    AboutScreen, ContactScreen, HomeScreen, LoginScreen, PostViewScreen, ProfileEditScreen, ProfileScreen, RegisterScreen,
};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AuthGuardContainer)]
    #[layout(NavBarContainer)]
    #[route("/")]
    HomeScreen {},

    #[route("/posts/:id")]
    PostViewScreen { id: i32 },

    #[route("/login")]
    LoginScreen {},

    #[route("/register")]
    RegisterScreen {},

    #[route("/profile")]
    ProfileScreen {},

    #[route("/profile/edit")]
    ProfileEditScreen {},

    #[route("/about")]
    AboutScreen {},

    #[route("/contact")]
    ContactScreen {},
}
