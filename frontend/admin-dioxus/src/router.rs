use dioxus::prelude::*;
use std::sync::LazyLock;

use crate::containers::AuthGuardContainer;
use crate::containers::NavBarContainer;

use crate::screens::AnalyticsScreen;
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
use crate::screens::SonnerDemoScreen;
use crate::screens::TagsAddScreen;
use crate::screens::TagsEditScreen;
use crate::screens::TagsListScreen;
use crate::screens::UsersAddScreen;
use crate::screens::UsersEditScreen;
use crate::screens::UsersListScreen;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AuthGuardContainer)]
    #[layout(NavBarContainer)]
    #[route("/")]
    HomeScreen {},

    #[route("/analytics")]
    AnalyticsScreen {},

    #[route("/login")]
    LoginScreen {},

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

    #[route("/users/add")]
    UsersAddScreen {},
    #[route("/users/:id/edit")]
    UsersEditScreen { id: i32 },
    #[route("/users")]
    UsersListScreen {},



    #[route("/demo/sonner")]
    SonnerDemoScreen {},
}
pub static OPEN_ROUTES: LazyLock<Vec<Route>> = LazyLock::new(|| vec![Route::LoginScreen {}]);
