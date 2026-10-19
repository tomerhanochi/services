//! Browsing, downloading, renaming and moving.

use crate::html::Html;
use authn_identity::prelude::Identity;
use axum::Form;
// Unlike axum's `Query`, reads repeated keys (`?item=a&item=b`) into a `Vec`.
use axum::body::Body;
use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Query as MultiQuery;
use file_server_core::prelude::{EntryKind, EntryName, FilesError, Location, MountName, RelPath};
use file_server_views::prelude::Href;
use file_server_views::prelude::{self as views, Done, Moving};
use htmlb::AttrValue;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use crate::auth::CurrentUser;
use crate::error::{bad_request, files_error};
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
fn sign_in(uri: &OriginalUri) -> Response {
    let next = uri.path_and_query().map_or("/", |p| p.as_str());
    let status = if next == "/" {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };
    (status, Html(views::signed_out(next))).into_response()
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
#[hotpath::measure(future = true)]
pub async fn home<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    match state.ports.files().mounts(&user).await {
        Ok(mounts) => Html(views::home(state.sign_out(&user), &mounts)).into_response(),
        Err(e) => files_error(e),
    }
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    done: Option<String>,
    /// Present (`?select`) for the listing in selection mode.
    select: Option<String>,
}

/// `GET /m/{mount}/{*path}`: a directory's listing, or a file's download.
#[hotpath::measure(future = true)]
pub async fn browse<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Query(query): Query<BrowseQuery>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    let result = async {
        let at = at.location()?;
        let files = state.ports.files();
        match files.metadata(&user, &at).await?.kind {
            EntryKind::Directory => {
                let dir = files.list(&user, &at).await?;
                if query.select.is_some() && dir.capabilities.moves {
                    return Ok(Html(views::select(&at, &dir.entries)).into_response());
                }
                let done = query.done.as_deref().and_then(Done::from_query);
                let page = views::directory(
                    state.sign_out(&user),
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
    result.await.unwrap_or_else(files_error)
}

#[hotpath::measure(future = true)]
async fn download<P: Ports>(
    state: &AppState<P>,
    user: &Identity,
    at: &Location,
) -> Result<Response, FilesError> {
    let (metadata, file) = state.ports.files().download(user, at).await?;
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
    Ok((
        headers,
        Body::from_stream(ReaderStream::new(hotpath::io!(
            file,
            label = "download-file"
        ))),
    )
        .into_response())
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
#[hotpath::measure(future = true)]
pub async fn rename_form<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
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
        Ok(Html(views::rename(&at, current, None)).into_response())
    };
    result.await.unwrap_or_else(files_error)
}

/// `GET /actions/{mount}/{*path}`: what the user can do with an entry.
#[hotpath::measure(future = true)]
pub async fn actions<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    let result = async {
        let at = at.location()?;
        at.path.name().ok_or(FilesError::Root)?;
        let files = state.ports.files();
        let capabilities = files.capabilities(&user, &at.mount).await?;
        if !(capabilities.rename || capabilities.moves) {
            return Err(if capabilities.list {
                FilesError::Forbidden
            } else {
                FilesError::NotFound
            });
        }
        let kind = files.metadata(&user, &at).await?.kind;
        Ok(Html(views::actions(&at, kind, capabilities)).into_response())
    };
    result.await.unwrap_or_else(files_error)
}

#[derive(Deserialize)]
pub struct RenameForm {
    name: String,
}

/// `POST /rename/{mount}/{*path}`
#[hotpath::measure(future = true)]
pub async fn rename<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Form(form): Form<RenameForm>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    let at = match at.location() {
        Ok(at) => at,
        Err(e) => return files_error(e),
    };
    let submitted = form.name.as_str();
    let invalid = |message: String| {
        let page = views::rename(&at, submitted, Some(&message));
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
        Err(e) => files_error(e),
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
#[hotpath::measure(future = true)]
pub async fn move_form<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Query(query): Query<MoveQuery>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
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
        picker(
            &state,
            &user,
            Moving::One(&item),
            &browsing,
            None,
            StatusCode::OK,
        )
        .await
    };
    result.await.unwrap_or_else(files_error)
}

#[hotpath::measure(future = true)]
async fn picker<P: Ports>(
    state: &AppState<P>,
    user: &Identity,
    moving: Moving<'_>,
    browsing: &Location,
    error: Option<&str>,
    status: StatusCode,
) -> Result<Response, FilesError> {
    let files = state.ports.files();
    let mount = match moving {
        Moving::One(item) => &item.mount,
        Moving::Many { from, .. } => &from.mount,
    };
    if !files.capabilities(user, mount).await?.moves {
        return Err(FilesError::Forbidden);
    }
    let folders = files.list(user, browsing).await?.entries;
    let page = views::move_picker(moving, browsing, &folders, error);
    Ok((status, Html(page)).into_response())
}

#[derive(Deserialize)]
pub struct MoveForm {
    to: String,
}

/// `POST /move/{mount}/{*path}`
#[hotpath::measure(future = true)]
pub async fn move_entry<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    Form(form): Form<MoveForm>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
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
                    Moving::One(&item),
                    &into,
                    Some(&message),
                    StatusCode::CONFLICT,
                )
                .await
            }
            Err(e) => Err(e),
        }
    };
    result.await.unwrap_or_else(files_error)
}

/// The most entries moved at once. Their names travel in the picker's URLs, which proxies
/// commonly cap at about 8 KB.
const MAX_SELECTION: usize = 200;

/// A `/move-many/` query: the entries' names, and the folder the picker is browsing.
#[derive(Deserialize)]
pub struct SelectionQuery {
    #[serde(default)]
    item: Vec<String>,
    to: Option<String>,
}

/// The entries to move: valid names, each once.
struct Selection {
    items: Vec<EntryName>,
    to: Option<String>,
}

impl Selection {
    fn new(query: SelectionQuery) -> Result<Selection, Response> {
        if query.item.len() > MAX_SELECTION {
            return Err(bad_request(
                "Too many selected",
                &format!("Move at most {MAX_SELECTION} items at once."),
            ));
        }
        let mut items = Vec::with_capacity(query.item.len());
        for name in query.item {
            let name = EntryName::new(name).map_err(|_| files_error(FilesError::NotFound))?;
            if !items.contains(&name) {
                items.push(name);
            }
        }
        if items.is_empty() {
            return Err(bad_request(
                "Nothing selected",
                "Select what to move first.",
            ));
        }
        Ok(Selection {
            items,
            to: query.to,
        })
    }
}

/// `GET /move-many/{mount}/{*path}?item=…&to=<folder>`: pick where to move entries of a
/// directory, starting there.
#[hotpath::measure(future = true)]
pub async fn move_many_form<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    MultiQuery(query): MultiQuery<SelectionQuery>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    let selection = match Selection::new(query) {
        Ok(selection) => selection,
        Err(response) => return response,
    };
    let result = async {
        let from = at.location()?;
        let browsing = match &selection.to {
            Some(to) => Location::new(
                from.mount.clone(),
                RelPath::parse(to).map_err(|_| FilesError::NotFound)?,
            ),
            None => from.clone(),
        };
        let moving = Moving::Many {
            from: &from,
            items: &selection.items,
        };
        picker(&state, &user, moving, &browsing, None, StatusCode::OK).await
    };
    result.await.unwrap_or_else(files_error)
}

/// `POST /move-many/{mount}/{*path}?item=…`: moves each entry, then shows the destination.
/// If some can't move, the picker comes back with just those.
#[hotpath::measure(future = true)]
pub async fn move_many<P: Ports>(
    State(state): State<AppState<P>>,
    CurrentUser(user): CurrentUser,
    uri: OriginalUri,
    Path(at): Path<At>,
    MultiQuery(query): MultiQuery<SelectionQuery>,
    Form(form): Form<MoveForm>,
) -> Response {
    let Some(user) = user else {
        return sign_in(&uri);
    };
    let selection = match Selection::new(query) {
        Ok(selection) => selection,
        Err(response) => return response,
    };
    let result = async {
        let from = at.location()?;
        let into = Location::new(
            from.mount.clone(),
            RelPath::parse(&form.to).map_err(|_| FilesError::NotFound)?,
        );
        let total = selection.items.len();
        let mut failed = Vec::new();
        let mut reason = None;
        for item in selection.items {
            let at = from.join(item.clone());
            match state.ports.files().move_into(&user, &at, &into).await {
                Ok(_) => {}
                Err(
                    e @ (FilesError::AlreadyExists
                    | FilesError::IntoItself
                    | FilesError::CrossDevice
                    | FilesError::NotFound),
                ) => {
                    reason.get_or_insert(e);
                    failed.push(item);
                }
                Err(e) => return Err(e),
            }
        }
        let Some(reason) = reason else {
            return Ok(Redirect::to(&listing_url(&into, Some(Done::Moved))).into_response());
        };
        let message = format!(
            "Moved {} of {total}. The other {}: {reason}.",
            total - failed.len(),
            failed.len(),
        );
        let moving = Moving::Many {
            from: &from,
            items: &failed,
        };
        picker(
            &state,
            &user,
            moving,
            &into,
            Some(&message),
            StatusCode::CONFLICT,
        )
        .await
    };
    result.await.unwrap_or_else(files_error)
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
