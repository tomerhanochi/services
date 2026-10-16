use crate::rejection::AuthnRejection;
use authn::prelude::{AuthOutcome, Authenticate};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

/// An authenticated identity, obtained using an authenticator from application state.
pub struct Authn<A: Authenticate>(pub A::Identity);

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
