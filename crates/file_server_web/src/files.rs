//! Browsing, downloading, renaming and moving.

use crate::html::Html;
use authn_identity::prelude::Identity;
use axum::Form;
use axum::body::Body;
use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use file_server_core::prelude::{EntryKind, EntryName, FilesError, Location, MountName, RelPath};
use file_server_views::prelude::Href;
use file_server_views::prelude::{self as views, Done};
use htmlb::AttrValue;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use crate::auth::CurrentUser;
use crate::error::files_error;
use crate::state::{AppState, Ports};

/// `{mount}` and the optional `{*path}` of a location route.
#[derive(Deserialize)]
pub struct At {
    mount: String,
    #[serde(default)]
    path: String,
}

impl At {
    /// Invalid names can't exist, so they're "not found".
    fn location(&self) -> Result<Location, FilesError> {
        let mount = MountName::new(self.mount.as_str()).map_err(|_| FilesError::NotFound)?;
        let path = RelPath::parse(&self.path).map_err(|_| FilesError::NotFound)?;
        Ok(Location::new(mount, path))
    }
}

/// Anonymous visitors see the sign-in page, which comes back here afterwards.
fn sign_in<P>(state: &AppState<P>, uri: &OriginalUri) -> Response {
    let next = uri.path_and_query().map_or("/", |p| p.as_str());
    let status = if next == "/" {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };
    (status, Html(views::signed_out(state.page(None), next))).into_response()
}

fn listing_url(dir: &Location, done: Option<Done>) -> String {
    let mut url = String::new();
    Href::dir(dir).write_value(&mut url);
    if let Some(done) = done {
        url.push_str("?done=");
        url.push_str(done.query());
    }
    url
}

/// `GET /`: the mounts the user can see.
pub async fn home<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    match state.ports.files().mounts(&user).await {
        Ok(mounts) => Html(views::home(state.page(Some(&user)), &mounts)).into_response(),
        Err(e) => files_error(&state, Some(&user), e),
    }
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    done: Option<String>,
}

/// `GET /m/{mount}/{*path}`: a directory's listing, or a file's download.
pub async fn browse<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Query(query): Query<BrowseQuery>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    let result = async {
        let at = at.location()?;
        let files = state.ports.files();
        match files.metadata(&user, &at).await?.kind {
            EntryKind::Directory => {
                let dir = files.list(&user, &at).await?;
                let done = query.done.as_deref().and_then(Done::from_query);
                let page = views::directory(
                    state.page(Some(&user)),
                    &at,
                    &dir.entries,
                    dir.capabilities,
                    done,
                );
                Ok(Html(page).into_response())
            }
            EntryKind::File => download(&state, &user, &at).await,
        }
    };
    result
        .await
        .unwrap_or_else(|e| files_error(&state, Some(&user), e))
}

async fn download<P: Ports>(
    state: &AppState<P>,
    user: &Identity,
    at: &Location,
) -> Result<Response, FilesError> {
    let (metadata, file) = state.ports.files().download(&user, at).await?;
    let name = at.path.name().map_or("download", EntryName::as_str);
    let headers = [
        (
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        ),
        (header::CONTENT_LENGTH, HeaderValue::from(metadata.size)),
        (header::CONTENT_DISPOSITION, content_disposition(name)),
        (
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        ),
    ];
    Ok((headers, Body::from_stream(ReaderStream::new(file))).into_response())
}

/// `attachment; filename="<ascii>"; filename*=UTF-8''<percent-encoded>` (RFC 6266): the
/// ASCII name for old clients, the exact one for everyone else.
fn content_disposition(name: &str) -> HeaderValue {
    let ascii: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut value = format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''");
    for b in name.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                value.push(b as char)
            }
            _ => value.push_str(&format!("%{b:02X}")),
        }
    }
    HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

/// `GET /rename/{mount}/{*path}`
pub async fn rename_form<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    let result = async {
        let at = at.location()?;
        let capabilities = state.ports.files().capabilities(&user, &at.mount).await?;
        if !capabilities.rename {
            return Err(if capabilities.list {
                FilesError::Forbidden
            } else {
                FilesError::NotFound
            });
        }
        state.ports.files().metadata(&user, &at).await?;
        let current = at.path.name().ok_or(FilesError::Root)?.as_str();
        Ok(Html(views::rename(state.page(Some(&user)), &at, current, None)).into_response())
    };
    result
        .await
        .unwrap_or_else(|e| files_error(&state, Some(&user), e))
}

#[derive(Deserialize)]
pub struct RenameForm {
    name: String,
}

/// `POST /rename/{mount}/{*path}`
pub async fn rename<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Form(form): Form<RenameForm>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    let at = match at.location() {
        Ok(at) => at,
        Err(e) => return files_error(&state, Some(&user), e),
    };
    let submitted = form.name.as_str();
    let invalid = |message: String| {
        let page = views::rename(state.page(Some(&user)), &at, submitted, Some(&message));
        (StatusCode::UNPROCESSABLE_ENTITY, Html(page)).into_response()
    };
    let name = match EntryName::new(submitted) {
        Ok(name) => name,
        Err(e) => return invalid(capitalize(&e.to_string())),
    };
    match state.ports.files().rename(&user, &at, name).await {
        Ok(renamed) => Redirect::to(&listing_url(
            &renamed.parent().unwrap_or(renamed.clone()),
            Some(Done::Renamed),
        ))
        .into_response(),
        Err(FilesError::AlreadyExists) => {
            invalid("Something with that name is already here.".into())
        }
        Err(e) => files_error(&state, Some(&user), e),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[derive(Deserialize)]
pub struct MoveQuery {
    to: Option<String>,
}

/// `GET /move/{mount}/{*path}?to=<folder>`: pick where to move to, starting where it is.
pub async fn move_form<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Query(query): Query<MoveQuery>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    let result = async {
        let item = at.location()?;
        let parent = item.parent().ok_or(FilesError::Root)?;
        let browsing = match query.to {
            Some(to) => Location::new(
                item.mount.clone(),
                RelPath::parse(&to).map_err(|_| FilesError::NotFound)?,
            ),
            None => parent,
        };
        picker(&state, &user, &item, &browsing, None, StatusCode::OK).await
    };
    result
        .await
        .unwrap_or_else(|e| files_error(&state, Some(&user), e))
}

async fn picker<P: Ports>(
    state: &AppState<P>,
    user: &Identity,
    item: &Location,
    browsing: &Location,
    error: Option<&str>,
    status: StatusCode,
) -> Result<Response, FilesError> {
    let files = state.ports.files();
    if !files.capabilities(&user, &item.mount).await?.moves {
        return Err(FilesError::Forbidden);
    }
    let folders = files.list(&user, browsing).await?.entries;
    let page = views::move_picker(state.page(Some(user)), item, browsing, &folders, error);
    Ok((status, Html(page)).into_response())
}

#[derive(Deserialize)]
pub struct MoveForm {
    to: String,
}

/// `POST /move/{mount}/{*path}`
pub async fn move_entry<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Form(form): Form<MoveForm>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&state, &uri);
    };
    let result = async {
        let item = at.location()?;
        let into = Location::new(
            item.mount.clone(),
            RelPath::parse(&form.to).map_err(|_| FilesError::NotFound)?,
        );
        match state.ports.files().move_into(&user, &item, &into).await {
            Ok(_) => Ok(Redirect::to(&listing_url(&into, Some(Done::Moved))).into_response()),
            Err(
                e @ (FilesError::AlreadyExists | FilesError::IntoItself | FilesError::CrossDevice),
            ) => {
                let message = capitalize(&e.to_string());
                picker(
                    &state,
                    &user,
                    &item,
                    &into,
                    Some(&message),
                    StatusCode::CONFLICT,
                )
                .await
            }
            Err(e) => Err(e),
        }
    };
    result
        .await
        .unwrap_or_else(|e| files_error(&state, Some(&user), e))
}

#[cfg(test)]
mod tests {
    use super::content_disposition;

    #[test]
    fn disposition_has_ascii_fallback_and_exact_name() {
        assert_eq!(
            content_disposition("résumé \"final\".pdf"),
            "attachment; filename=\"r_sum_ _final_.pdf\"; filename*=UTF-8''r%C3%A9sum%C3%A9%20%22final%22.pdf"
        );
    }
}
