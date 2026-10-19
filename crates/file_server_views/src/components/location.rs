use file_server_core::prelude::{EntryName, Location, MountName};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::icons::Icons;
use crate::urls::MountHref;

/// The last component of a location's path, or its mount's name at the root.
pub fn location_title(at: &Location) -> &str {
    at.path.name().map_or(at.mount.as_str(), EntryName::as_str)
}

/// `photos / 2024 / trip`: a location's full path.
#[derive(Clone, Copy)]
pub struct Breadcrumb<'a>(pub &'a Location);

impl htmlb::Component for Breadcrumb<'_> {
    fn render(self) -> impl IntoHtml {
        path(self.0.mount.as_str(), self.0.path.components())
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

/// The folders above a location, for under a headline that already names it. Empty at a
/// mount's root (and the subtitle is hidden when empty).
#[derive(Clone, Copy)]
pub struct Ancestors<'a>(pub &'a Location);

impl htmlb::Component for Ancestors<'_> {
    fn render(self) -> impl IntoHtml {
        let components = self.0.path.components();
        components
            .split_last()
            .map(|(_, above)| path(self.0.mount.as_str(), above))
    }
    fn dynamic_len_hint(&self) -> usize {
        Breadcrumb(self.0).dynamic_len_hint()
    }
}

fn path(mount: &str, components: &[EntryName]) -> impl IntoHtml {
    (
        mount,
        components
            .iter()
            .map(|name| (" / ", name.as_str()))
            .into_html_cloned(),
    )
}

/// A mount on the home page, opening its root listing.
pub fn mount_row(mount: &MountName) -> impl IntoHtml {
    list_item(mount.as_str())
        .leading(Icons::HardDrive)
        .href(MountHref(mount))
        .trailing_text(Icons::ChevronRight)
}
