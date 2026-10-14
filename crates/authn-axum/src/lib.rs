use std::fmt;

use authn::{AuthOutcome, Authenticate};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

/// An authenticated identity, obtained using an authenticator from application state.
pub struct Authn<A: Authenticate>(pub A::Identity);

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

impl<S, A> FromRequestParts<S> for Authn<A>
where
    S: Send + Sync,
    A: Authenticate<Context = Parts> + FromRef<S> + Send + Sync,
    A::Identity: Send,
{
    type Rejection = AuthnRejection<A::Error>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let authenticator = A::from_ref(state);

        match authenticator.authenticate(parts).await {
            Ok(AuthOutcome::Authenticated(identity)) => Ok(Self(identity)),
            Ok(AuthOutcome::NotApplicable | AuthOutcome::Rejected) => {
                Err(AuthnRejection::Unauthorized)
            }
            Err(error) => Err(AuthnRejection::Authentication(error)),
        }
    }
}
