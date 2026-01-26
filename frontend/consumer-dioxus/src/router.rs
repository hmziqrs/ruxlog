use dioxus::prelude::*;
use crate::containers::{AuthGuardContainer, NavBarContainer};
use crate::screens::{
    AboutScreen, AdvertiseScreen, CategoriesScreen, CategoryDetailScreen, ContactScreen, HomeScreen,
    LoginScreen, PostViewScreen, PrivacyPolicyScreen, TagDetailScreen, TagsScreen, TermsScreen,
};

#[cfg(feature = "auth-register")]
use crate::screens::RegisterScreen;

#[cfg(feature = "profile-management")]
use crate::screens::{ProfileEditScreen, ProfileScreen};

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

    #[cfg(feature = "auth-register")]
    #[route("/register")]
    RegisterScreen {},

    #[cfg(feature = "profile-management")]
    #[route("/profile")]
    ProfileScreen {},

    #[cfg(feature = "profile-management")]
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
