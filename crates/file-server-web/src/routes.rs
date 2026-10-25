use crate::{
    auth, files, security,
    state::{AppState, Ports},
};
use authn_oidc_axum::prelude::{
    OidcHttp, callback as oidc_callback, login as oidc_login, logout as oidc_logout,
};
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, header},
    middleware::{from_fn, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use file_server_views::prelude::Asset;
use file_server_views::prelude::Route;
use tower_http::compression::CompressionLayer;

pub fn router<P: Ports>(state: AppState<P>, oidc: OidcHttp) -> Router {
    let origin = state.public_origin.clone();
    let mut router = hotpath::axum!(
        Router::new()
            .route(Route::Home.path(), get(files::home::<P>))
            .route(Route::BrowseMount.path(), get(files::browse::<P>))
            .route(Route::BrowseRoot.path(), get(files::browse::<P>))
            .route(Route::BrowsePath.path(), get(files::browse::<P>))
            .route(
                Route::Rename.path(),
                get(files::rename_form::<P>).post(files::rename::<P>),
            )
            .route(
                Route::Move.path(),
                get(files::move_form::<P>).post(files::move_entry::<P>),
            )
            .route(Route::Actions.path(), get(files::actions::<P>))
            .route(
                Route::MoveManyMount.path(),
                get(files::move_many_form::<P>).post(files::move_many::<P>),
            )
            .route(
                Route::MoveManyPath.path(),
                get(files::move_many_form::<P>).post(files::move_many::<P>),
            )
            .route(Route::Login.path(), get(auth::login::<P>))
            .route(Route::Logout.path(), post(auth::logout::<P>))
            .route(Route::Health.path(), get(|| async { "ok" }))
            .route(Asset::Css.url(), get(css::<P>))
            .route(Asset::Favicon.url(), get(favicon))
            .with_state(state)
    );
    let oidc_routes = Router::new()
        .route(Route::OidcLogin.path(), get(oidc_login))
        .route(Route::OidcCallback.path(), get(oidc_callback))
        .route(Route::OidcLogout.path(), post(oidc_logout))
        .with_state(oidc);
    router = router.merge(oidc_routes);
    hotpath::axum!(
        router
            .layer(from_fn_with_state(origin, security::same_origin))
            .layer(from_fn(security::security_headers))
            .layer(CompressionLayer::new())
            .layer(from_fn(security::log_request))
    )
}
async fn css<P: Ports>(State(state): State<AppState<P>>) -> Response {
    asset("text/css; charset=utf-8", state.css.as_str().to_owned())
}
async fn favicon() -> Response {
    asset("image/svg+xml", include_str!("favicon.svg"))
}
fn asset(content_type: &'static str, body: impl IntoResponse) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    response
}
