use std::{error::Error, fmt};

use authn::{AuthOutcome, Authenticate};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

/// An authenticated identity, obtained using an authenticator from application state.
pub struct Authn<A: Authenticate>(pub A::Identity);

/// Extracts one Authorization credential for `scheme` (case insensitive).
/// A matching but malformed or ambiguous header returns an empty credential,
/// allowing the verifier to reject it rather than treating it as absent.
pub fn authorization<'a>(parts: &'a Parts, scheme: &str) -> Option<&'a [u8]> {
    let values = parts.headers.get_all(axum::http::header::AUTHORIZATION);
    let count = values.iter().count();
    for value in values {
        let bytes = value.as_bytes();
        let end = bytes
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(bytes.len());
        if !bytes[..end].eq_ignore_ascii_case(scheme.as_bytes()) {
            continue;
        }
        if count != 1 || bytes.get(end) != Some(&b' ') {
            return Some(&[]);
        }
        let credential = &bytes[end..];
        let start = credential
            .iter()
            .position(|byte| *byte != b' ')
            .unwrap_or(credential.len());
        return Some(&credential[start..]);
    }
    None
}

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
