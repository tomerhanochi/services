use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{error::Error, fmt};
/// Authentication rejection with the original operational error available for diagnostics.
#[derive(Debug)]
pub enum AuthnRejection<E> {
    Unauthorized,
    Authentication(E),
}

impl<E: fmt::Display> fmt::Display for AuthnRejection<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => f.write_str("authentication required"),
            Self::Authentication(error) => error.fmt(f),
        }
    }
}

impl<E> IntoResponse for AuthnRejection<E> {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Self::Authentication(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl<E: Error + 'static> Error for AuthnRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unauthorized => None,
            Self::Authentication(error) => Some(error),
        }
    }
}
