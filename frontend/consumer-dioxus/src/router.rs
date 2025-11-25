use dioxus::prelude::*;
use crate::containers::{AuthGuardContainer, NavBarContainer};
use crate::screens::{
    AboutScreen, AdvertiseScreen, CategoriesScreen, CategoryDetailScreen, ContactScreen, HomeScreen,
    LoginScreen, PostViewScreen, PrivacyPolicyScreen, ProfileEditScreen, ProfileScreen,
    RegisterScreen, TagDetailScreen, TagsScreen, TermsScreen,
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

    #[route("/privacy")]
    PrivacyPolicyScreen {},

    #[route("/terms")]
    TermsScreen {},

    #[route("/advertise")]
    AdvertiseScreen {},
}
