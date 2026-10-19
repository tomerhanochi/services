use file_server_core::prelude::Location;
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{Breadcrumb, close_button};
use crate::icons::Icons;
use crate::page::page;
use crate::urls::ParentHref;

/// The rename form: a full-screen dialog, with the action in the bottom bar where a thumb is.
#[hotpath::measure]
pub fn rename(at: &Location, current: &str, error: Option<&str>) -> impl IntoHtml {
    let bar = top_app_bar("Rename")
        .navigation(close_button("Cancel", ParentHref::of(at)))
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
            .icon(Icons::Check)
            .submit_form("rename"),
    );
    page()
        .title("Rename")
        .content(scaffold(bar, form).bottom_app_bar(actions))
}
