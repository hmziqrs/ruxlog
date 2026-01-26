use dioxus::prelude::*;
use std::sync::LazyLock;

use crate::containers::AuthGuardContainer;
use crate::containers::NavBarContainer;

use crate::screens::CategoriesAddScreen;
use crate::screens::CategoriesEditScreen;
use crate::screens::CategoriesListScreen;
use crate::screens::HomeScreen;
use crate::screens::LoginScreen;
use crate::screens::MediaListScreen;
use crate::screens::MediaUploadScreen;
use crate::screens::PostsAddScreen;
use crate::screens::PostsEditScreen;
use crate::screens::PostsListScreen;
use crate::screens::PostsViewScreen;
use crate::screens::ProfileSecurityScreen;
use crate::screens::SonnerDemoScreen;
use crate::screens::TagsAddScreen;
use crate::screens::TagsEditScreen;
use crate::screens::TagsListScreen;

#[cfg(feature = "analytics")]
use crate::screens::AnalyticsScreen;

#[cfg(feature = "comments")]
use crate::screens::{CommentsListScreen, FlaggedCommentsScreen};

#[cfg(feature = "newsletter")]
use crate::screens::{NewsletterSendScreen, NewsletterSubscribersScreen};

#[cfg(feature = "admin-routes")]
use crate::screens::RoutesSettingsScreen;

#[cfg(feature = "admin-acl")]
use crate::screens::AclSettingsScreen;

#[cfg(feature = "user-management")]
use crate::screens::{UsersAddScreen, UsersEditScreen, UsersListScreen};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AuthGuardContainer)]
    #[layout(NavBarContainer)]
    #[route("/")]
    HomeScreen {},

    #[route("/login")]
    LoginScreen {},

    #[cfg(feature = "analytics")]
    #[route("/analytics")]
    AnalyticsScreen {},

    #[cfg(feature = "comments")]
    #[route("/comments")]
    CommentsListScreen {},

    #[cfg(feature = "comments")]
    #[route("/comments/flagged")]
    FlaggedCommentsScreen {},

    #[cfg(feature = "newsletter")]
    #[route("/newsletter/subscribers")]
    NewsletterSubscribersScreen {},

    #[cfg(feature = "newsletter")]
    #[route("/newsletter/send")]
    NewsletterSendScreen {},

    #[cfg(feature = "admin-routes")]
    #[route("/settings/routes")]
    RoutesSettingsScreen {},

    #[cfg(feature = "admin-acl")]
    #[route("/settings/acl")]
    AclSettingsScreen {},

    #[route("/profile/security")]
    ProfileSecurityScreen {},

    #[route("/posts/add")]
    PostsAddScreen {},
    #[route("/posts/:id/edit")]
    PostsEditScreen { id: i32 },
    #[route("/posts/:id")]
    PostsViewScreen { id: i32 },
    #[route("/posts")]
    PostsListScreen {},

    #[route("/categories/add")]
    CategoriesAddScreen {},
    #[route("/categories")]
    CategoriesListScreen {},
    #[route("/categories/:id/edit")]
    CategoriesEditScreen { id: i32 },

    #[route("/tags/add")]
    TagsAddScreen {},
    #[route("/tags/:id/edit")]
    TagsEditScreen { id: i32 },
    #[route("/tags")]
    TagsListScreen {},

    #[route("/media/upload")]
    MediaUploadScreen {},
    #[route("/media")]
    MediaListScreen {},

    #[cfg(feature = "user-management")]
    #[route("/users/add")]
    UsersAddScreen {},

    #[cfg(feature = "user-management")]
    #[route("/users/:id/edit")]
    UsersEditScreen { id: i32 },

    #[cfg(feature = "user-management")]
    #[route("/users")]
    UsersListScreen {},

    #[route("/demo/sonner")]
    SonnerDemoScreen {},
}
pub static OPEN_ROUTES: LazyLock<Vec<Route>> = LazyLock::new(|| vec![Route::LoginScreen {}]);
