//! Every URL the pages link to. The router in `file_server_web` serves the same shapes.

use std::fmt::Write;

use file_server_core::prelude::{EntryName, Location, RelPath};
use htmlb::AttrValue;

pub const HOME: &str = "/";
pub const LOGIN: &str = "/login";
pub const LOGOUT: &str = "/logout";
pub const AUTH_CALLBACK: &str = "/auth/callback";
pub const ASSETS: &str = "/assets";

/// What a location URL leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// `/m/{mount}/{path}`: a directory's listing (with a trailing slash) or a file's download.
    Browse,
    /// `/rename/{mount}/{path}`
    Rename,
    /// `/move/{mount}/{path}`
    Move,
}

impl Route {
    pub const fn prefix(self) -> &'static str {
        match self {
            Route::Browse => "/m/",
            Route::Rename => "/rename/",
            Route::Move => "/move/",
        }
    }
}

/// A URL to a location, percent-encoded straight into the output (no `String` per link).
/// Encoded URLs contain only `A-Z a-z 0-9 - . _ ~ / % ? =`, so they need no HTML escaping.
#[derive(Clone, Copy)]
pub struct Href<'a> {
    route: Route,
    location: &'a Location,
    child: Option<&'a EntryName>,
    directory: bool,
    /// `?to=<path>` for the move picker.
    to: Option<&'a RelPath>,
}

impl<'a> Href<'a> {
    pub fn new(route: Route, location: &'a Location) -> Self {
        Href {
            route,
            location,
            child: None,
            directory: false,
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
    const VALUE_MIN_LEN: usize = 3;

    fn value_len_hint(&self) -> usize {
        self.route.prefix().len()
            + self.location.mount.as_str().len()
            + self.components().map(|c| c.len() + 1).sum::<usize>()
            + 1
    }

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
        if let Some(to) = self.to {
            out.push_str("?to=");
            for (i, component) in to.components().iter().enumerate() {
                if i > 0 {
                    out.push_str("%2F");
                }
                encode(out, component.as_str());
            }
        }
    }
}

/// `/login?next=<path>`, `next` already being a path on this site.
pub struct LoginHref<'a>(pub &'a str);

impl AttrValue for LoginHref<'_> {
    const VALUE_MIN_LEN: usize = LOGIN.len();

    fn value_len_hint(&self) -> usize {
        LOGIN.len() + 6 + self.0.len()
    }

    fn write_value(self, out: &mut String) {
        out.push_str(LOGIN);
        if !self.0.is_empty() && self.0 != HOME {
            out.push_str("?next=");
            encode(out, self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use file_server_core::prelude::MountName;
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
        assert_eq!(
            a().href(Href::dir(&Location::root(MountName::new("m").unwrap())))
                .to_html(),
            r#"<a href="/m/m/"></a>"#
        );
    }
}
