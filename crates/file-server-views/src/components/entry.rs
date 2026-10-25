use file_server_core::prelude::{Capabilities, Entry, EntryKind, Location, Metadata};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use super::{Date, Size};
use crate::icons::Icons;
use crate::urls::{Href, Route};

/// `Folder`, or `1.5 MB · 2024-05-01`.
#[derive(Clone, Copy)]
pub struct Details(pub Metadata);

impl Component for Details {
    fn render(self) -> impl IntoHtml {
        match self.0.kind {
            EntryKind::Directory => Either::Left("Folder"),
            EntryKind::File => {
                Either::Right((Size(self.0.size), self.0.modified.map(|m| (" · ", Date(m)))))
            }
        }
    }
}
impl RunTimeHtmlSizeHint for Details {
    fn size_hint(&self) -> usize {
        match self.0.kind {
            EntryKind::Directory => "Folder".len(),
            EntryKind::File => {
                Size(self.0.size).size_hint()
                    + self
                        .0
                        .modified
                        .map_or(0, |m| " · ".len() + Date(m).size_hint())
            }
        }
    }
}

/// An entry of the listing of `at`: a folder opens, a file downloads.
pub fn entry_row<'a>(at: &'a Location, entry: &'a Entry, can: Capabilities) -> impl IntoHtml {
    let is_dir = entry.metadata.kind == EntryKind::Directory;
    let href = Href::new(Route::Browse, at)
        .child(&entry.name)
        .directory(is_dir);
    let name = entry.name.as_str();
    let item = list_item()
        .headline(name)
        .leading(if is_dir { Icons::Folder } else { Icons::Draft })
        .supporting(Details(entry.metadata))
        .action(match is_dir {
            true => Either::Left(Link(href)),
            false => Either::Right(Download(href)),
        });
    // A link to the actions page, not a menu in the row: a listing would otherwise
    // repeat the same menu once per entry, about half of its bytes.
    let actions = (can.rename || can.moves).then(|| {
        icon_button()
            .icon(Icons::MoreVert)
            .label(("Actions for ", name))
            .href(Href::new(Route::Actions, at).child(&entry.name))
    });
    match actions {
        Some(actions) => Either::Left(item.trailing(actions)),
        None => Either::Right(item),
    }
}

/// What the user can do with the entry at `at`, as list items.
pub fn entry_actions(at: &Location, kind: EntryKind, can: Capabilities) -> impl IntoHtml {
    (
        (kind == EntryKind::File && can.download).then(|| {
            list_item()
                .headline("Download")
                .leading(Icons::Download)
                .download(Href::new(Route::Browse, at))
        }),
        can.rename.then(|| {
            list_item()
                .headline("Rename")
                .leading(Icons::Edit)
                .href(Href::new(Route::Rename, at))
        }),
        // Without `?to=`, the picker starts in the entry's own folder.
        can.moves.then(|| {
            list_item()
                .headline("Move")
                .leading(Icons::DriveFileMove)
                .href(Href::new(Route::Move, at))
        }),
    )
}
