use file_server_core::prelude::{Capabilities, EntryKind, Location};
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{Breadcrumb, close_button, entry_actions, location_title};
use crate::page::page;
use crate::urls::ParentHref;

/// What can be done with the entry at `at`, reached from its row's menu button.
#[hotpath::measure]
pub fn actions(at: &Location, kind: EntryKind, can: Capabilities) -> impl IntoHtml {
    let name = location_title(at);
    let bar = top_app_bar(name)
        .navigation(close_button("Close", ParentHref::of(at)))
        .subtitle(Breadcrumb(at));
    page()
        .title(name)
        .content(scaffold(bar, list(entry_actions(at, kind, can))))
}
