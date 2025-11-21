use dioxus::prelude::*;
use std::sync::LazyLock;
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

pub static OPEN_ROUTES: LazyLock<Vec<Route>> = LazyLock::new(|| vec![
    Route::LoginScreen {},
    Route::RegisterScreen {},
    Route::HomeScreen {},
    Route::PostViewScreen { id: 0 }, // Note: Pattern matching might be needed for dynamic routes in real auth guard
    Route::AboutScreen {},
    Route::ContactScreen {},
]);
