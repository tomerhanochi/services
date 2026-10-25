//! Every URL the pages link to. The router in `file_server_web` serves the same shapes.

use std::fmt::Write;

use file_server_core::prelude::{EntryName, Location, MountName, RelPath};
use htmlb::{AttrValue, CompileTimeHtmlSizeHint, RunTimeHtmlSizeHint};

pub const HOME: &str = "/";
pub const LOGIN: &str = "/login";
pub const LOGOUT: &str = "/logout";
pub const AUTH_CALLBACK: &str = "/auth/callback";

/// A file every page links to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asset {
    Css,
    Favicon,
}

impl Asset {
    pub const fn url(self) -> &'static str {
        match self {
            Asset::Css => "/assets/app.css",
            Asset::Favicon => "/assets/favicon.svg",
        }
    }
}

/// What a location URL leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// `/m/{mount}/{path}`: a directory's listing (with a trailing slash) or a file's download.
    Browse,
    /// `/rename/{mount}/{path}`
    Rename,
    /// `/move/{mount}/{path}`
    Move,
    /// `/actions/{mount}/{path}`: what can be done with an entry.
    Actions,
    /// `/move-many/{mount}/{path}?item=…&item=…`: moving entries of a directory.
    MoveMany,
}

impl Route {
    pub const fn prefix(self) -> &'static str {
        match self {
            Route::Browse => "/m/",
            Route::Rename => "/rename/",
            Route::Move => "/move/",
            Route::Actions => "/actions/",
            Route::MoveMany => "/move-many/",
        }
    }
}

/// A URL to a location, percent-encoded straight into the output (no `String` per link).
/// Encoded URLs contain only `A-Z a-z 0-9 - . _ ~ / % ? = &`, so they need no HTML escaping:
/// an `&` followed by a name and `=`, not `;`, is never a character reference.
#[derive(Clone, Copy)]
pub struct Href<'a> {
    route: Route,
    location: &'a Location,
    child: Option<&'a EntryName>,
    directory: bool,
    /// `?item=<name>&…` for moving several entries.
    items: &'a [EntryName],
    /// `?to=<path>` for the move picker. Always last.
    to: Option<&'a RelPath>,
}

impl<'a> Href<'a> {
    pub fn new(route: Route, location: &'a Location) -> Self {
        Href {
            route,
            location,
            child: None,
            directory: false,
            items: &[],
            to: None,
        }
    }

    /// A directory's listing: ends in `/`, so relative behaviour matches the filesystem.
    pub fn dir(location: &'a Location) -> Self {
        Href {
            directory: true,
            ..Href::new(Route::Browse, location)
        }
    }

    /// `location/child`, without building that location.
    pub fn child(mut self, child: &'a EntryName) -> Self {
        self.child = Some(child);
        self
    }

    pub fn directory(mut self, directory: bool) -> Self {
        self.directory = directory;
        self
    }

    /// The entries being moved, by name.
    pub fn items(mut self, items: &'a [EntryName]) -> Self {
        self.items = items;
        self
    }

    /// The move picker, browsing `to`.
    pub fn to(mut self, to: &'a RelPath) -> Self {
        self.to = Some(to);
        self
    }

    fn components(&self) -> impl Iterator<Item = &'a str> {
        self.location
            .path
            .components()
            .iter()
            .chain(self.child)
            .map(EntryName::as_str)
    }
}

/// Percent-encodes everything but RFC 3986's unreserved characters.
fn encode(out: &mut String, s: &str) {
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
}

impl AttrValue for Href<'_> {
    fn write_value(self, out: &mut String) {
        out.push_str(self.route.prefix());
        out.push_str(self.location.mount.as_str());
        for component in self.components() {
            out.push('/');
            encode(out, component);
        }
        if self.directory {
            out.push('/');
        }
        let mut separator = '?';
        for item in self.items {
            out.push(separator);
            out.push_str("item=");
            encode(out, item.as_str());
            separator = '&';
        }
        if let Some(to) = self.to {
            out.push(separator);
            out.push_str("to=");
            for (i, component) in to.components().iter().enumerate() {
                if i > 0 {
                    out.push_str("%2F");
                }
                encode(out, component.as_str());
            }
        }
    }
}

impl CompileTimeHtmlSizeHint for Href<'_> {
    const SIZE_HINT: usize = 3;
}
impl RunTimeHtmlSizeHint for Href<'_> {
    fn size_hint(&self) -> usize {
        (self.route.prefix().len()
            + self.location.mount.as_str().len()
            + self.components().map(|c| c.len() + 1).sum::<usize>()
            + self
                .items
                .iter()
                .map(|i| i.as_str().len() + 6)
                .sum::<usize>()
            + 1)
        .saturating_sub(Self::SIZE_HINT)
    }
}

/// `/login?next=<path>`, `next` already being a path on this site.
pub struct LoginHref<'a>(pub &'a str);

impl AttrValue for LoginHref<'_> {
    fn write_value(self, out: &mut String) {
        out.push_str(LOGIN);
        if !self.0.is_empty() && self.0 != HOME {
            out.push_str("?next=");
            encode(out, self.0);
        }
    }
}
impl CompileTimeHtmlSizeHint for LoginHref<'_> {
    const SIZE_HINT: usize = LOGIN.len();
}
impl RunTimeHtmlSizeHint for LoginHref<'_> {
    fn size_hint(&self) -> usize {
        LOGIN.len() + 6 + self.0.len() - Self::SIZE_HINT
    }
}

/// `/m/{mount}/`, without building a `Location`.
#[derive(Clone, Copy)]
pub struct MountHref<'a>(pub &'a MountName);

impl AttrValue for MountHref<'_> {
    fn write_value(self, out: &mut String) {
        out.push_str(Route::Browse.prefix());
        out.push_str(self.0.as_str()); // `[a-z0-9_-]`: nothing to encode.
        out.push('/');
    }
}
impl CompileTimeHtmlSizeHint for MountHref<'_> {
    const SIZE_HINT: usize = 4;
}
impl RunTimeHtmlSizeHint for MountHref<'_> {
    fn size_hint(&self) -> usize {
        self.0.as_str().len()
    }
}

/// A directory's listing URL; owns the `Location` it was given.
pub struct ParentHref(pub Location);

impl ParentHref {
    /// The listing `at` is in, or `at` itself at a mount's root.
    pub fn of(at: &Location) -> ParentHref {
        ParentHref(at.parent().unwrap_or_else(|| at.clone()))
    }
}

impl AttrValue for ParentHref {
    fn write_value(self, out: &mut String) {
        Href::dir(&self.0).write_value(out)
    }
}
impl CompileTimeHtmlSizeHint for ParentHref {
    const SIZE_HINT: usize = 3;
}
impl RunTimeHtmlSizeHint for ParentHref {
    fn size_hint(&self) -> usize {
        Href::dir(&self.0).size_hint()
    }
}

/// The listing in selection mode: `?select`.
#[derive(Clone, Copy)]
pub struct SelectHref<'a>(pub &'a Location);

impl AttrValue for SelectHref<'_> {
    fn write_value(self, out: &mut String) {
        Href::dir(self.0).write_value(out);
        out.push_str("?select");
    }
}
impl CompileTimeHtmlSizeHint for SelectHref<'_> {
    const SIZE_HINT: usize = 10;
}
impl RunTimeHtmlSizeHint for SelectHref<'_> {
    fn size_hint(&self) -> usize {
        Href::dir(self.0).size_hint() + 7
    }
}

/// The picker browsing `parent`.
pub struct OwnedPicker<'a>(pub Href<'a>, pub RelPath);

impl AttrValue for OwnedPicker<'_> {
    fn write_value(self, out: &mut String) {
        self.0.to(&self.1).write_value(out)
    }
}
impl CompileTimeHtmlSizeHint for OwnedPicker<'_> {
    const SIZE_HINT: usize = 3;
}
impl RunTimeHtmlSizeHint for OwnedPicker<'_> {
    fn size_hint(&self) -> usize {
        self.0.to(&self.1).size_hint()
    }
}

/// The picker browsing a child of the current folder.
#[derive(Clone, Copy)]
pub struct PickerChild<'a>(pub Href<'a>, pub &'a EntryName);

impl AttrValue for PickerChild<'_> {
    fn write_value(self, out: &mut String) {
        self.0.write_value(out);
        // `to` already ends the URL; append `/child` (or the first component).
        out.push_str(if out.ends_with("to=") { "" } else { "%2F" });
        encode(out, self.1.as_str());
    }
}
impl CompileTimeHtmlSizeHint for PickerChild<'_> {
    const SIZE_HINT: usize = 3;
}
impl RunTimeHtmlSizeHint for PickerChild<'_> {
    fn size_hint(&self) -> usize {
        self.0.size_hint() + 3 + self.1.as_str().len()
    }
}

#[cfg(test)]
mod tests {
    use htmlb::prelude::*;

    use super::*;

    #[test]
    fn hrefs_are_encoded() {
        let location = Location::new(
            MountName::new("photos").unwrap(),
            RelPath::parse("a b/ü\"<").unwrap(),
        );
        let name = EntryName::new("x&y.jpg").unwrap();
        let html = a()
            .href(Href::new(Route::Browse, &location).child(&name))
            .to_html();
        assert_eq!(
            html,
            r#"<a href="/m/photos/a%20b/%C3%BC%22%3C/x%26y.jpg"></a>"#
        );
        let to = RelPath::parse("p/q").unwrap();
        let html = a()
            .href(Href::new(Route::Move, &location).to(&to))
            .to_html();
        assert_eq!(
            html,
            r#"<a href="/move/photos/a%20b/%C3%BC%22%3C?to=p%2Fq"></a>"#
        );
        let items = [EntryName::new("a b").unwrap(), EntryName::new("c").unwrap()];
        let html = a()
            .href(Href::new(Route::MoveMany, &location).items(&items).to(&to))
            .to_html();
        assert_eq!(
            html,
            r#"<a href="/move-many/photos/a%20b/%C3%BC%22%3C?item=a%20b&item=c&to=p%2Fq"></a>"#
        );
        assert_eq!(
            a().href(Href::dir(&Location::root(MountName::new("m").unwrap())))
                .to_html(),
            r#"<a href="/m/m/"></a>"#
        );
    }
}
