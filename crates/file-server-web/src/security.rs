use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub(crate) async fn same_origin(
    State(origin): State<String>,
    request: Request,
    next: Next,
) -> Response {
    if !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    ) {
        if request
            .headers()
            .get("sec-fetch-site")
            .is_some_and(|site| site == "cross-site")
        {
            tracing::warn!(
                method = %request.method(),
                path = request.uri().path(),
                reason = "cross_site_fetch",
                "request blocked by same-origin guard"
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        let supplied = request.headers().get(header::ORIGIN);
        if supplied.is_some_and(|supplied| supplied.as_bytes() != origin.as_bytes())
            || (request.headers().contains_key(header::COOKIE) && supplied.is_none())
        {
            tracing::warn!(
                method = %request.method(),
                path = request.uri().path(),
                reason = if supplied.is_some() { "origin_mismatch" } else { "missing_origin_with_cookie" },
                "request blocked by same-origin guard"
            );
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    next.run(request).await
}

pub(crate) async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let response = next.run(request).await;
    let status = response.status();
    tracing::info!(%method, %path, %status, "HTTP request completed");
    response
}

pub(crate) async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("same-origin"));
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; script-src 'none'; style-src 'self' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; base-uri 'none'; form-action 'self'; frame-ancestors 'none'"));
    headers
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("private, no-store"));
    response
}
