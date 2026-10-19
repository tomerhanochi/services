use file_server_core::prelude::{Entry, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use super::Details;
use crate::icons::Icons;
use crate::urls::{Href, Route};

/// The id of the form the checkboxes submit with: it's in the bottom bar, not around the
/// list, so the rows stay plain list items.
const SELECTION: &str = "selection";

/// An entry with a checkbox: the whole row toggles it.
pub fn selectable_row(entry: &Entry) -> impl IntoHtml {
    let name = entry.name.as_str();
    list_item(name)
        .leading(checkbox("item", name).form(SELECTION))
        .supporting(Details(entry.metadata))
        .label()
}

/// The bar moving the [`selectable_row`]s checked in `at`, shown once one is.
pub fn selection_bar(at: &Location) -> impl IntoHtml {
    // A GET form: the picker keeps the selection in its URL as the user browses.
    htmlb_components::selection_bar(
        form()
            .id(SELECTION)
            .method(FormMethod::Get)
            .action(Href::new(Route::MoveMany, at))
            .child(
                filled_button("Move selected")
                    .icon(Icons::DriveFileMove)
                    .submit(),
            ),
    )
}
