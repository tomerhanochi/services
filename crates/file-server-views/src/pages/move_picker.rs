use file_server_core::prelude::{Entry, EntryKind, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{
    Breadcrumb, Moving, close_button, error_message, move_here, picker_folder_row, picker_up_row,
};
use crate::page::page;

/// The move destination picker, showing the folders in `browsing`.
#[hotpath::measure]
pub fn move_picker<'a>(
    moving: Moving<'a>,
    browsing: &'a Location,
    folders: &'a [Entry],
    error: Option<&str>,
) -> impl IntoHtml {
    let bar = top_app_bar(moving.title())
        .navigation(close_button("Cancel", moving.back()))
        .subtitle(("To ", Breadcrumb(browsing)));

    // What's being moved isn't offered as a destination.
    let rows = folders
        .iter()
        .filter(|f| f.metadata.kind == EntryKind::Directory)
        .filter(move |f| !moving.includes(browsing, &f.name))
        .map(move |f| picker_folder_row(moving, browsing, &f.name));
    let body = (
        error.map(error_message),
        list((picker_up_row(moving, browsing), rows.into_html_cloned())),
    );
    page()
        .title(moving.title())
        .content(scaffold(bar, body).bottom_app_bar(bottom_app_bar(move_here(moving, browsing))))
}
