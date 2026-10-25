use file_server_core::prelude::{Capabilities, Entry, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{
    Ancestors, Done, empty, entry_row, location_title, select_button, sign_out, up_button,
};
use crate::page::page;

/// A directory's listing. `user` is the signed-in user's name, when they can sign out.
#[hotpath::measure]
pub fn directory<'a>(
    user: Option<&str>,
    at: &'a Location,
    entries: &'a [Entry],
    can: Capabilities,
    done: Option<Done>,
) -> impl IntoHtml {
    let page_title = location_title(at);
    let bar = top_app_bar(page_title)
        .navigation(up_button(at))
        .subtitle(Ancestors(at))
        .actions((
            (can.moves && !entries.is_empty()).then(|| select_button(at)),
            sign_out(user),
        ));

    let rows = entries.iter().map(move |entry| entry_row(at, entry, can));
    let body = match entries.is_empty() {
        true => Either::Left(empty("This folder is empty.")),
        false => Either::Right(list(rows.into_html_cloned())),
    };
    page()
        .title(page_title)
        .content(scaffold(bar, body).snackbar(done.map(Done::message)))
}
