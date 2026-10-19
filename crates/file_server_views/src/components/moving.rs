use file_server_core::prelude::{EntryName, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use super::location_title;
use crate::icons::Icons;
use crate::urls::{Href, OwnedPicker, ParentHref, PickerChild, Route};

/// What the picker moves.
#[derive(Clone, Copy)]
pub enum Moving<'a> {
    One(&'a Location),
    /// Entries of the directory `from`, by name.
    Many {
        from: &'a Location,
        items: &'a [EntryName],
    },
}

impl<'a> Moving<'a> {
    /// The picker's URL without `?to=`. Its form posts here.
    pub fn href(self) -> Href<'a> {
        match self {
            Moving::One(item) => Href::new(Route::Move, item),
            Moving::Many { from, items } => Href::new(Route::MoveMany, from).items(items),
        }
    }

    pub fn title(self) -> impl IntoHtml {
        match self {
            Moving::One(item) => Either::Left(("Move ", location_title(item))),
            Moving::Many { items: [item], .. } => Either::Left(("Move ", item.as_str())),
            Moving::Many { items, .. } => Either::Right(("Move ", items.len(), " items")),
        }
    }

    /// Where cancelling goes: the listing the move started from.
    pub fn back(self) -> ParentHref {
        match self {
            Moving::One(item) => ParentHref::of(item),
            Moving::Many { from, .. } => ParentHref(from.clone()),
        }
    }

    /// Whether `folder`, in `browsing`, is being moved: a folder can't move into itself.
    pub fn includes(self, browsing: &Location, folder: &EntryName) -> bool {
        match self {
            Moving::One(item) => browsing.join(folder.clone()) == *item,
            Moving::Many { from, items } => browsing == from && items.contains(folder),
        }
    }
}

/// The picker browsing the folder above `browsing`; none at a mount's root.
pub fn picker_up_row(moving: Moving, browsing: &Location) -> impl IntoHtml {
    browsing.path.parent().map(|parent| {
        list_item("Up one folder")
            .leading(Icons::ArrowUpward)
            .href(OwnedPicker(moving.href(), parent))
    })
}

/// The picker browsing `folder`, in `browsing`.
pub fn picker_folder_row<'a>(
    moving: Moving<'a>,
    browsing: &'a Location,
    folder: &'a EntryName,
) -> impl IntoHtml {
    list_item(folder.as_str())
        .leading(Icons::Folder)
        .trailing_text(Icons::ChevronRight)
        .href(PickerChild(moving.href().to(&browsing.path), folder))
}

/// Moves into `browsing`, for the picker's bottom bar.
pub fn move_here(moving: Moving, browsing: &Location) -> impl IntoHtml {
    form()
        .method(FormMethod::Post)
        .action(moving.href())
        .child((
            input()
                .r#type(InputType::Hidden)
                .name("to")
                .value(htmlb::display(&browsing.path)),
            filled_button("Move here")
                .icon(Icons::DriveFileMove)
                .submit(),
        ))
}
