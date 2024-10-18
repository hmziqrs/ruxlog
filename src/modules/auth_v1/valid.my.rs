use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
    Json,
};
use axum_valid::{Valid, ValidRejection};
use serde_json::json;
use validator::ValidationErrors;

pub struct ValidError<T>(pub Valid<T>);

impl<T> std::ops::Deref for ValidError<Valid<T>> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ValidError<Valid<T>> {
    pub fn into_inner(self) -> T {
        self.0.into_inner().into_inner()
    }
}
