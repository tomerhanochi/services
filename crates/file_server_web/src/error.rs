//! Failures, as pages.

use authn_identity::prelude::Identity;
use std::fmt::Display;

use crate::html::Html;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use file_server_core::prelude::FilesError;

use crate::state::AppState;

/// Logs the cause; shows the user only that it's not their fault.
pub fn internal<P>(state: &AppState<P>, user: Option<&Identity>, error: impl Display) -> Response {
    tracing::error!(%error, "internal error");
    let page = file_server_views::prelude::error(
        state.page(user),
        "Something went wrong",
        "That's on our side. Try again in a moment.",
    );
    (StatusCode::INTERNAL_SERVER_ERROR, Html(page)).into_response()
}

/// A file operation's failure.
pub fn files_error<P>(state: &AppState<P>, user: Option<&Identity>, error: FilesError) -> Response {
    let status = match &error {
        FilesError::NotFound => StatusCode::NOT_FOUND,
        FilesError::Forbidden => StatusCode::FORBIDDEN,
        FilesError::Internal(_) => return internal(state, user, error),
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
        Html(file_server_views::prelude::error(
            state.page(user),
            title,
            &message,
        )),
    )
        .into_response()
}
