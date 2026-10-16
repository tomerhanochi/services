use crate::state::{AppState, Ports};
use authn::prelude::{AuthOutcome, Authenticate};
use authn_identity::prelude::Identity;
use axum::{
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Redirect, Response},
};

pub(crate) struct CurrentUser(pub Option<Identity>);
impl<P: Ports> FromRequestParts<AppState<P>> for CurrentUser {
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, state: &AppState<P>) -> Result<Self, Response> {
        match state.ports.authenticator().authenticate(parts).await {
            Ok(AuthOutcome::Authenticated(identity)) => Ok(Self(Some(identity))),
            Ok(AuthOutcome::NotApplicable) => Ok(Self(None)),
            Ok(AuthOutcome::Rejected) => Err(StatusCode::UNAUTHORIZED.into_response()),
            Err(error) => {
                tracing::error!(%error, "authentication failed");
                Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}
pub(crate) async fn login<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
) -> Response {
    if user.is_some() {
        return Redirect::to("/").into_response();
    }
    if state.oidc_enabled {
        return Redirect::to("/auth/login").into_response();
    }
    (
        StatusCode::UNAUTHORIZED,
        [(
            header::WWW_AUTHENTICATE,
            r#"Basic realm="Files", charset="UTF-8""#,
        )],
        "Authentication required",
    )
        .into_response()
}
pub(crate) async fn logout<P: Ports>(State(state): State<AppState<P>>) -> Response {
    if state.oidc_enabled {
        Redirect::temporary("/auth/logout").into_response()
    } else {
        Redirect::to("/").into_response()
    }
}
