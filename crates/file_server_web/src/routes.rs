use crate::{
    auth, files, security,
    state::{AppState, Ports},
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
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub fn router<P: Ports>(state: AppState<P>, auth_routes: Router) -> Router {
    let origin = state.public_origin.clone();
    hotpath::axum!(
        Router::new()
            .route("/", get(files::home::<P>))
            .route("/m/{mount}", get(files::browse::<P>))
            .route("/m/{mount}/", get(files::browse::<P>))
            .route("/m/{mount}/{*path}", get(files::browse::<P>))
            .route(
                "/rename/{mount}/{*path}",
                get(files::rename_form::<P>).post(files::rename::<P>),
            )
            .route(
                "/move/{mount}/{*path}",
                get(files::move_form::<P>).post(files::move_entry::<P>),
            )
            .route("/actions/{mount}/{*path}", get(files::actions::<P>))
            .route(
                "/move-many/{mount}",
                get(files::move_many_form::<P>).post(files::move_many::<P>),
            )
            .route(
                "/move-many/{mount}/{*path}",
                get(files::move_many_form::<P>).post(files::move_many::<P>),
            )
            .route("/login", get(auth::login::<P>))
            .route("/logout", post(auth::logout::<P>))
            .route("/healthz", get(|| async { "ok" }))
            .route(Asset::Css.url(), get(css::<P>))
            .route(Asset::Favicon.url(), get(favicon))
            .with_state(state)
            .nest("/auth", auth_routes)
            .layer(from_fn_with_state(origin, security::same_origin))
            .layer(from_fn(security::security_headers))
            .layer(CompressionLayer::new())
            .layer(TraceLayer::new_for_http())
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
