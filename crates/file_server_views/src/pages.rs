use crate::urls;
use file_server_core::prelude::{
    Capabilities, Entry, EntryKind, EntryName, Location, Metadata, MountName, RelPath,
};
use htmlb::prelude::*;
use htmlb_components::action::{Download, Link};
use htmlb_components::prelude::*;

use crate::format::{Date, Size};
use crate::urls::{Href, LoginHref, Route};

/// What every page needs.
#[derive(Clone, Copy)]
pub struct Page<'a> {
    /// The theme's and the components' stylesheet URLs.
    pub stylesheet: &'a str,
    /// The signed-in user's display name.
    pub user: Option<&'a str>,
    pub logout: bool,
}

impl<'a> Page<'a> {
    fn document<T: IntoHtml + 'a, B: IntoHtml + 'a>(
        self,
        page_title: T,
        body: B,
    ) -> impl IntoHtml + 'a {
        document((page_title, " · Files"), self.stylesheet.to_owned(), body)
    }

    /// Sign out, as a form: logging out changes state, so it's a POST.
    fn sign_out(self) -> impl IntoHtml + 'a {
        self.user.filter(|_| self.logout).map(|name| {
            form()
                .method(FormMethod::Post)
                .action(urls::LOGOUT)
                .child(icon_button(icon::Logout, ("Sign out ", name)).submit())
        })
    }
}

/// What just happened, shown as a snackbar after a redirect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    Renamed,
    Moved,
}

impl Done {
    pub const fn query(self) -> &'static str {
        match self {
            Done::Renamed => "renamed",
            Done::Moved => "moved",
        }
    }

    pub fn from_query(s: &str) -> Option<Done> {
        match s {
            "renamed" => Some(Done::Renamed),
            "moved" => Some(Done::Moved),
            _ => None,
        }
    }

    fn message(self) -> &'static str {
        match self {
            Done::Renamed => "Renamed",
            Done::Moved => "Moved",
        }
    }
}

/// For visitors who aren't signed in.
pub fn signed_out<'a>(page: Page<'a>, next: &'a str) -> impl IntoHtml + 'a {
    let body = (
        headline("Your files, on your server"),
        supporting_text("Sign in to browse, download, rename and move them."),
        filled_button("Sign in")
            .icon(icon::Login)
            .href(LoginHref(next)),
    );
    page.document("Sign in", scaffold(top_app_bar("Files"), pane(body)))
}

/// The mounts the user can see.
pub fn home<'a>(page: Page<'a>, mounts: &'a [MountName]) -> impl IntoHtml + 'a {
    let rows = mounts.iter().map(|mount| {
        list_item(mount.as_str())
            .leading(icon::HardDrive)
            .href(MountHref(mount))
            .trailing_text(icon::ChevronRight)
    });
    let body = match mounts.is_empty() {
        true => Either::Left(empty(
            "No folders are shared with you yet. Ask the server's administrator to give one of your roles access.",
        )),
        false => Either::Right(list(rows.into_html_cloned())),
    };
    page.document(
        "Home",
        scaffold(top_app_bar("Files").actions(page.sign_out()), body),
    )
}

/// `/m/{mount}/`, without building a `Location`.
#[derive(Clone, Copy)]
struct MountHref<'a>(&'a MountName);

impl htmlb::AttrValue for MountHref<'_> {
    const VALUE_MIN_LEN: usize = 4;
    fn value_len_hint(&self) -> usize {
        4 + self.0.as_str().len()
    }
    fn write_value(self, out: &mut String) {
        out.push_str(Route::Browse.prefix());
        out.push_str(self.0.as_str()); // `[a-z0-9_-]`: nothing to encode.
        out.push('/');
    }
}

/// A directory's listing.
pub fn directory<'a>(
    page: Page<'a>,
    at: &'a Location,
    entries: &'a [Entry],
    can: Capabilities,
    done: Option<Done>,
) -> impl IntoHtml + 'a {
    let page_title = location_title(at);
    let up = match at.parent() {
        Some(parent) => {
            Either::Left(icon_button(icon::ArrowBack, "Up one folder").href(ParentHref(parent)))
        }
        None => Either::Right(icon_button(icon::ArrowBack, "All folders").href(urls::HOME)),
    };
    let bar = top_app_bar(page_title)
        .navigation(up)
        .subtitle(Breadcrumb::ancestors(at))
        .actions(page.sign_out());

    let rows = entries
        .iter()
        .enumerate()
        .map(move |(i, entry)| row(at, i, entry, can));
    let body = match entries.is_empty() {
        true => Either::Left(empty("This folder is empty.")),
        false => Either::Right(list(rows.into_html_cloned())),
    };
    page.document(
        page_title,
        scaffold(bar, body).snackbar(done.map(Done::message)),
    )
}

/// A parent directory's listing URL; owns the `Location` it was given.
struct ParentHref(Location);

impl htmlb::AttrValue for ParentHref {
    const VALUE_MIN_LEN: usize = 3;
    fn value_len_hint(&self) -> usize {
        Href::dir(&self.0).value_len_hint()
    }
    fn write_value(self, out: &mut String) {
        Href::dir(&self.0).write_value(out)
    }
}

fn location_title(at: &Location) -> &str {
    at.path.name().map_or(at.mount.as_str(), EntryName::as_str)
}

/// `photos / 2024 / trip`: a location's full path.
#[derive(Clone, Copy)]
struct Breadcrumb<'a>(&'a Location);

impl<'a> Breadcrumb<'a> {
    /// The folders above `location`, for under a headline that already names it. Empty at
    /// a mount's root (and the subtitle is hidden when empty).
    fn ancestors(location: &'a Location) -> Ancestors<'a> {
        Ancestors(location)
    }

    fn render_components(mount: &'a str, components: &'a [EntryName]) -> impl IntoHtml + 'a {
        (
            mount,
            components
                .iter()
                .map(|name| (" / ", name.as_str()))
                .into_html_cloned(),
        )
    }
}

impl htmlb::Component for Breadcrumb<'_> {
    fn render(self) -> impl IntoHtml {
        Breadcrumb::render_components(self.0.mount.as_str(), self.0.path.components())
    }
    fn dynamic_len_hint(&self) -> usize {
        self.0.mount.as_str().len()
            + self
                .0
                .path
                .components()
                .iter()
                .map(|c| c.as_str().len() + 3)
                .sum::<usize>()
    }
}

#[derive(Clone, Copy)]
struct Ancestors<'a>(&'a Location);

impl htmlb::Component for Ancestors<'_> {
    fn render(self) -> impl IntoHtml {
        let components = self.0.path.components();
        components
            .split_last()
            .map(|(_, above)| Breadcrumb::render_components(self.0.mount.as_str(), above))
    }
    fn dynamic_len_hint(&self) -> usize {
        Breadcrumb(self.0).dynamic_len_hint()
    }
}

fn row<'a>(at: &'a Location, i: usize, entry: &'a Entry, can: Capabilities) -> impl IntoHtml + 'a {
    let is_dir = entry.metadata.kind == EntryKind::Directory;
    let href = Href::new(Route::Browse, at)
        .child(&entry.name)
        .directory(is_dir);
    let name = entry.name.as_str();
    let item = list_item(name)
        .leading(if is_dir {
            Either::Left(icon::Folder)
        } else {
            Either::Right(icon::Draft)
        })
        .supporting(Details(entry.metadata))
        .action(match is_dir {
            true => Either::Left(Link(href)),
            false => Either::Right(Download(href)),
        });
    let actions = (can.rename || can.moves).then(|| {
        let sheet_id = ("actions-", i);
        (
            icon_button(icon::MoreVert, ("Actions for ", name)).opens(sheet_id),
            sheet(sheet_id, name).child((
                (!is_dir && can.download)
                    .then(|| menu_item(icon::Download, "Download").download(href)),
                can.rename.then(|| {
                    menu_item(icon::Edit, "Rename")
                        .href(Href::new(Route::Rename, at).child(&entry.name))
                }),
                can.moves.then(|| {
                    menu_item(icon::DriveFileMove, "Move")
                        .href(Href::new(Route::Move, at).child(&entry.name).to(&at.path))
                }),
            )),
        )
    });
    match actions {
        Some(actions) => Either::Left(item.trailing(actions)),
        None => Either::Right(item),
    }
}

/// `Folder`, or `1.5 MB · 2024-05-01`.
#[derive(Clone, Copy)]
struct Details(Metadata);

impl htmlb::Component for Details {
    fn render(self) -> impl IntoHtml {
        match self.0.kind {
            EntryKind::Directory => Either::Left("Folder"),
            EntryKind::File => Either::Right((
                htmlb::display(Size(self.0.size)),
                self.0.modified.map(|m| (" · ", htmlb::display(Date(m)))),
            )),
        }
    }
    fn dynamic_len_hint(&self) -> usize {
        // "1.5 MB · 2024-05-01"
        20
    }
}

fn empty(message: &str) -> impl IntoHtml + '_ {
    pane(supporting_text(message))
}

/// The rename form: a full-screen dialog, with the action in the bottom bar where a thumb is.
pub fn rename<'a>(
    page: Page<'a>,
    at: &'a Location,
    current: &'a str,
    error: Option<&'a str>,
) -> impl IntoHtml + 'a {
    let back = ParentHref(at.parent().unwrap_or_else(|| at.clone()));
    let bar = top_app_bar("Rename")
        .navigation(icon_button(icon::Close, "Cancel").href(back))
        .subtitle(Breadcrumb(at));
    let field = text_field("Name", "name")
        .value(current)
        .required()
        .autofocus()
        .verbatim()
        .error(error.is_some())
        .supporting(error.unwrap_or("Letters, numbers, spaces and symbols except /"));
    let form = pane(form().id("rename").method(FormMethod::Post).child(field));
    let actions = bottom_app_bar(
        filled_button("Rename")
            .icon(icon::Check)
            .submit_form("rename"),
    );
    page.document("Rename", scaffold(bar, form).bottom_app_bar(actions))
}

/// The move destination picker, showing the folders in `browsing`.
pub fn move_picker<'a>(
    page: Page<'a>,
    item: &'a Location,
    browsing: &'a Location,
    folders: &'a [Entry],
    error: Option<&'a str>,
) -> impl IntoHtml + 'a {
    let name = location_title(item);
    let close = ParentHref(item.parent().unwrap_or_else(|| item.clone()));
    let bar = top_app_bar(("Move ", name))
        .navigation(icon_button(icon::Close, "Cancel").href(close))
        .subtitle(("To ", Breadcrumb(browsing)));

    let picker = |to: &'a RelPath| Href::new(Route::Move, item).to(to);
    let up = browsing.path.parent().map(|parent| {
        list_item("Up one folder")
            .leading(icon::ArrowUpward)
            .href(OwnedPicker(Href::new(Route::Move, item), parent))
    });
    // A folder can't be moved into itself, so it isn't offered as a destination.
    let rows = folders
        .iter()
        .filter(|f| f.metadata.kind == EntryKind::Directory)
        .filter(move |f| browsing.join(f.name.clone()) != *item)
        .map(move |f| {
            list_item(f.name.as_str())
                .leading(icon::Folder)
                .trailing_text(icon::ChevronRight)
                .href(PickerChild(picker(&browsing.path), &f.name))
        });
    let body = (
        error.map(|e| pane(p().class("md-color-error").child(e))),
        list((up, rows.into_html_cloned())),
    );
    let action = form().method(FormMethod::Post).child((
        input()
            .r#type(InputType::Hidden)
            .name("to")
            .value(htmlb::display(&browsing.path)),
        filled_button("Move here")
            .icon(icon::DriveFileMove)
            .submit(),
    ));
    page.document(
        ("Move ", name),
        scaffold(bar, body).bottom_app_bar(bottom_app_bar(action)),
    )
}

/// The picker browsing `parent`.
struct OwnedPicker<'a>(Href<'a>, RelPath);

impl htmlb::AttrValue for OwnedPicker<'_> {
    const VALUE_MIN_LEN: usize = 3;
    fn value_len_hint(&self) -> usize {
        self.0.to(&self.1).value_len_hint()
    }
    fn write_value(self, out: &mut String) {
        self.0.to(&self.1).write_value(out)
    }
}

/// The picker browsing a child of the current folder.
#[derive(Clone, Copy)]
struct PickerChild<'a>(Href<'a>, &'a EntryName);

impl htmlb::AttrValue for PickerChild<'_> {
    const VALUE_MIN_LEN: usize = 3;
    fn value_len_hint(&self) -> usize {
        self.0.value_len_hint() + 3 + self.1.as_str().len()
    }
    fn write_value(self, out: &mut String) {
        self.0.write_value(out);
        // `to` already ends the URL; append `/child` (or the first component).
        out.push_str(if out.ends_with("?to=") { "" } else { "%2F" });
        for b in self.1.as_str().bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    out.push(b as char)
                }
                _ => {
                    use std::fmt::Write;
                    let _ = write!(out, "%{b:02X}");
                }
            }
        }
    }
}

/// Something went wrong, with a way back.
pub fn error<'a>(page: Page<'a>, page_title: &'a str, message: &'a str) -> impl IntoHtml + 'a {
    let bar =
        top_app_bar(page_title).navigation(icon_button(icon::Home, "All folders").href(urls::HOME));
    page.document(page_title, scaffold(bar, empty(message)))
}
