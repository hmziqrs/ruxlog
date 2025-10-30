use std::ops::{Deref, DerefMut};

use axum::extract::multipart::MultipartRejection;
use axum::extract::{FromRequest, Multipart, Request};

use crate::error::ErrorResponse;

/// Wrapper around axum's Multipart that converts rejections into our ErrorResponse
#[derive(Debug)]
pub struct ValidatedMultipart(pub Multipart);

impl<S> FromRequest<S> for ValidatedMultipart
where
    S: Send + Sync + 'static,
    Multipart: FromRequest<S, Rejection = MultipartRejection>,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Multipart::from_request(req, state).await {
            Ok(m) => Ok(Self(m)),
            Err(err) => Err(err.into()),
        }
    }
}

impl Deref for ValidatedMultipart {
    type Target = Multipart;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ValidatedMultipart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
