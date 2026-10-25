use crate::state::{AppState, Ports};
use authn::prelude::{AuthOutcome, Authenticate};
use authn_identity::prelude::Identity;
use axum::{
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use file_server_views::prelude::Route;

pub(crate) struct CurrentUser(pub Identity);
impl<P: Ports> FromRequestParts<AppState<P>> for CurrentUser {
    type Rejection = Response;
    #[hotpath::measure(future = true)]
    async fn from_request_parts(parts: &mut Parts, state: &AppState<P>) -> Result<Self, Response> {
        match state.ports.authenticator().authenticate(parts).await {
            Ok(AuthOutcome::Authenticated(identity)) => Ok(Self(identity)),
            Ok(AuthOutcome::NotApplicable | AuthOutcome::Rejected) => {
                Err(Redirect::to(Route::OidcLogin.path()).into_response())
            }
            Err(error) => {
                let report = error_stack::Report::new(error);
                tracing::error!(
                    component = "authentication",
                    error = %format!("{report:#}"),
                    "authentication failed"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}

#[hotpath::measure(future = true)]
pub(crate) async fn login<P: Ports>(CurrentUser(_user): CurrentUser) -> Response {
    Redirect::to(Route::Home.path()).into_response()
}
#[hotpath::measure(future = true)]
pub(crate) async fn logout<P: Ports>(State(_state): State<AppState<P>>) -> Response {
    Redirect::temporary(Route::OidcLogout.path()).into_response()
}
