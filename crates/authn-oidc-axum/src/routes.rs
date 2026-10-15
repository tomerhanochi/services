use std::{error::Error, fmt, time::Duration};

use authn_oidc::{OidcClient, OidcError, SessionId};
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

use crate::CookieSettings;

/// Routes are relative: `/login`, `/callback`, and POST `/logout`.
#[derive(Debug, Clone)]
pub struct OidcHttp {
    client: OidcClient,
    cookies: CookieSettings,
    success_path: String,
}

impl OidcHttp {
    pub fn new(
        client: OidcClient,
        cookies: CookieSettings,
        success_path: impl Into<String>,
    ) -> Result<Self, OidcError> {
        let success_path = success_path.into();
        if !success_path.starts_with('/')
            || success_path.starts_with("//")
            || success_path.contains('\\')
            || HeaderValue::from_str(&success_path).is_err()
        {
            return Err(OidcError::Configuration(
                "login success path must be a local absolute path",
            ));
        }
        Ok(Self {
            client,
            cookies,
            success_path,
        })
    }

    pub fn router<S: Clone + Send + Sync + 'static>(&self) -> Router<S> {
        Router::new()
            .route("/login", get(login))
            .route("/callback", get(callback))
            .route("/logout", post(logout))
            .with_state(self.clone())
    }
}

#[derive(Debug)]
pub struct OidcHttpError(pub OidcError);

impl From<OidcError> for OidcHttpError {
    fn from(error: OidcError) -> Self {
        Self(error)
    }
}

impl fmt::Display for OidcHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for OidcHttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl IntoResponse for OidcHttpError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            OidcError::InvalidCallback => StatusCode::BAD_REQUEST,
            OidcError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            OidcError::Provider(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
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

async fn callback(
    State(http): State<OidcHttp>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Result<Response, OidcHttpError> {
    if query.error.is_some() {
        return Err(OidcError::InvalidCallback.into());
    }
    let code = query.code.as_deref().ok_or(OidcError::InvalidCallback)?;
    let state = query.state.as_deref().ok_or(OidcError::InvalidCallback)?;
    let browser_state = http
        .cookies
        .login_value(&headers)
        .ok_or(OidcError::InvalidCallback)?;
    let session = http
        .client
        .complete_login(code, state, browser_state)
        .await?;
    // Replacing a session also revokes the previous browser session.
    if let Some(previous) = http.cookies.session_value(&headers) {
        http.client
            .sessions()
            .revoke(&SessionId::from_cookie(previous))?;
    }
    let mut response = Redirect::to(&http.success_path).into_response();
    set_cookie(
        &mut response,
        http.cookies
            .session_header(session.expose_secret(), http.client.session_ttl()),
    )?;
    set_cookie(&mut response, http.cookies.login_header("", Duration::ZERO))?;
    private_response(&mut response);
    Ok(response)
}

async fn logout(
    State(http): State<OidcHttp>,
    headers: HeaderMap,
) -> Result<Response, OidcHttpError> {
    if let Some(session) = http.cookies.session_value(&headers) {
        http.client
            .sessions()
            .revoke(&SessionId::from_cookie(session))?;
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
