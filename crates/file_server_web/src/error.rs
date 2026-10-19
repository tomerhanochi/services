//! Failures, as pages.

use std::fmt::Display;

use crate::html::Html;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use file_server_core::prelude::FilesError;

/// Logs the cause; shows the user only that it's not their fault.
pub fn internal(error: impl Display) -> Response {
    tracing::error!(%error, "internal error");
    let page = file_server_views::prelude::error(
        "Something went wrong",
        "That's on our side. Try again in a moment.",
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(page)).into_response()
}

/// A request the pages wouldn't have made, e.g. a hand-edited URL.
pub fn bad_request(title: &str, message: &str) -> Response {
    let page = file_server_views::prelude::error(title, message);
    (StatusCode::BAD_REQUEST, Html(page)).into_response()
}

/// A file operation's failure.
pub fn files_error(error: FilesError) -> Response {
    let status = match &error {
        FilesError::NotFound => StatusCode::NOT_FOUND,
        FilesError::Forbidden => StatusCode::FORBIDDEN,
        FilesError::Internal(_) => return internal(error),
        _ => StatusCode::CONFLICT,
    };
    let message = error.to_string();
    let title = if status == StatusCode::NOT_FOUND {
        "Not found"
    } else {
        "Can't do that"
    };
    (
        status,
        Html(file_server_views::prelude::error(title, &message)),
    )
        .into_response()
}
