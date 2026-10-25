use file_server_core::prelude::{Entry, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{Breadcrumb, close_button, selectable_row, selection_bar};
use crate::page::page;
use crate::urls::Href;

/// A directory's listing with a checkbox per entry, for moving several at once. Rows
/// toggle instead of opening; the bar to move them appears once one is checked.
#[hotpath::measure]
pub fn select(at: &Location, entries: &[Entry]) -> impl IntoHtml {
    let bar = top_app_bar()
        .headline("Select")
        .navigation(close_button("Done selecting", Href::dir(at)))
        .subtitle(Breadcrumb(at));
    let rows = entries.iter().map(selectable_row);
    page().title("Select").content(
        scaffold()
            .top_app_bar(bar)
            .body(list().items(rows.into_html_cloned()))
            .bottom_app_bar(selection_bar(at)),
    )
}
