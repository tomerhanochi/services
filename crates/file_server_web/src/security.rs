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
            return StatusCode::FORBIDDEN.into_response();
        }
        let supplied = request.headers().get(header::ORIGIN);
        if supplied.is_some_and(|supplied| supplied.as_bytes() != origin.as_bytes())
            || (request.headers().contains_key(header::COOKIE) && supplied.is_none())
        {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    next.run(request).await
}
pub(crate) async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; script-src 'none'; style-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'"));
    headers
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("private, no-store"));
    response
}
