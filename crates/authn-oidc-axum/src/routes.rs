use std::time::Duration;

use authn_oidc::prelude::{OidcCallbackError, OidcClient, OidcError, SessionId};
use axum::{
    Router,
    extract::{Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CACHE_CONTROL, SET_COOKIE},
    },
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;

use crate::prelude::{CookieSettings, SuccessPath};
use authn::prelude::Extracted;

/// Routes are relative: `/login`, `/callback`, and POST `/logout`.
#[derive(Debug, Clone)]
pub struct OidcHttp {
    client: OidcClient,
    cookies: CookieSettings,
    success_path: SuccessPath,
}

impl OidcHttp {
    pub fn new(client: OidcClient, cookies: CookieSettings, success_path: SuccessPath) -> Self {
        Self {
            client,
            cookies,
            success_path,
        }
    }

    pub fn router<S: Clone + Send + Sync + 'static>(&self) -> Router<S> {
        Router::new()
            .route("/login", get(login))
            .route("/callback", get(callback))
            .route("/logout", post(logout))
            .with_state(self.clone())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("OIDC HTTP request")]
struct OidcHttpContext;

pub struct OidcHttpError {
    status: StatusCode,
    report: error_stack::Report<OidcHttpContext>,
}

impl From<OidcError> for OidcHttpError {
    fn from(error: OidcError) -> Self {
        let status = match &error {
            OidcError::Callback(_) => StatusCode::BAD_REQUEST,
            OidcError::InvalidSessionToken | OidcError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            OidcError::Provider(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            report: error_stack::Report::new(error).change_context(OidcHttpContext),
        }
    }
}

impl From<OidcCallbackError> for OidcHttpError {
    fn from(error: OidcCallbackError) -> Self {
        Self::from(OidcError::from(error))
    }
}

impl IntoResponse for OidcHttpError {
    fn into_response(self) -> Response {
        let status = self.status;
        if status.is_server_error() {
            tracing::error!(status = %status, error = %format!("{:#}", self.report), "OIDC request failed");
        } else {
            tracing::warn!(status = %status, error = %format!("{:#}", self.report), "OIDC request failed");
        }
        let mut response = status.into_response();
        private_response(&mut response);
        response
    }
}

fn private_response(response: &mut Response) {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
}

fn set_cookie(response: &mut Response, cookie: String) -> Result<(), OidcHttpError> {
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|_| OidcError::Configuration("invalid OIDC cookie value"))?,
    );
    Ok(())
}

#[hotpath::measure(future = true)]
async fn login(State(http): State<OidcHttp>) -> Result<Response, OidcHttpError> {
    let login = http.client.begin_login()?;
    let mut response = Redirect::to(&login.url).into_response();
    set_cookie(
        &mut response,
        http.cookies
            .login_header(login.state.secret(), http.client.login_ttl()),
    )?;
    private_response(&mut response);
    Ok(response)
}

#[derive(Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[hotpath::measure(future = true)]
async fn callback(
    State(http): State<OidcHttp>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Result<Response, OidcHttpError> {
    if query.error.is_some() {
        return Err(OidcCallbackError::ProviderRejected.into());
    }
    let Some(code) = query.code.as_deref() else {
        return Err(OidcCallbackError::MissingAuthorizationCode.into());
    };
    let Some(state) = query.state.as_deref() else {
        return Err(OidcCallbackError::MissingState.into());
    };
    let browser_state = match http.cookies.login_value(&headers) {
        Extracted::Present(state) => state,
        Extracted::Absent => return Err(OidcCallbackError::MissingLoginStateCookie.into()),
        Extracted::Malformed => return Err(OidcCallbackError::MalformedLoginStateCookie.into()),
    };
    let session = http
        .client
        .complete_login(code, state, browser_state)
        .await?;
    // Replacing a session also revokes the previous browser session.
    if let Extracted::Present(previous) = http
        .cookies
        .session_value(&headers)
        .parse(SessionId::from_cookie)
    {
        http.client.sessions().revoke(&previous)?;
    }
    let mut response = Redirect::to(http.success_path.as_str()).into_response();
    set_cookie(
        &mut response,
        http.cookies
            .session_header(session.expose_secret(), http.client.session_ttl()),
    )?;
    set_cookie(&mut response, http.cookies.login_header("", Duration::ZERO))?;
    private_response(&mut response);
    Ok(response)
}

#[hotpath::measure(future = true)]
async fn logout(
    State(http): State<OidcHttp>,
    headers: HeaderMap,
) -> Result<Response, OidcHttpError> {
    if let Extracted::Present(session) = http
        .cookies
        .session_value(&headers)
        .parse(SessionId::from_cookie)
    {
        http.client.sessions().revoke(&session)?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    set_cookie(
        &mut response,
        http.cookies.session_header("", Duration::ZERO),
    )?;
    set_cookie(&mut response, http.cookies.login_header("", Duration::ZERO))?;
    private_response(&mut response);
    Ok(response)
}
