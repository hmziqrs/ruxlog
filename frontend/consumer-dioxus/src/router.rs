use dioxus::prelude::*;
use crate::containers::{AuthGuardContainer, NavBarContainer};
use crate::screens::{
    AboutScreen, CategoriesScreen, CategoryDetailScreen, ContactScreen, HomeScreen, LoginScreen,
    PostViewScreen, ProfileEditScreen, ProfileScreen, RegisterScreen, TagDetailScreen, TagsScreen,
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

    #[route("/tags")]
    TagsScreen {},

    #[route("/tags/:slug")]
    TagDetailScreen { slug: String },

    #[route("/categories")]
    CategoriesScreen {},

    #[route("/categories/:slug")]
    CategoryDetailScreen { slug: String },

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
