use authn::{AuthOutcome, Authenticate};
use axum::{extract::{FromRef, FromRequestParts}, http::{StatusCode, request::Parts}};

pub struct Authn<A: Authenticate>(A::Identity);

impl<S, A> FromRequestParts<S> for Authn<A>
where
    S: Send + Sync,
    A: Authenticate<Context = Parts> + FromRef<S> + Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let authenticator = A::from_ref(state);

        match authenticator.authenticate(parts).await {
            Ok(outcome) => match outcome {
                AuthOutcome::Authenticated(identity) => Ok(Self(identity)),
                AuthOutcome::NotApplicable => Err(StatusCode::UNAUTHORIZED),
                AuthOutcome::Rejected => Err(StatusCode::UNAUTHORIZED),
            },
            Err(err) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
