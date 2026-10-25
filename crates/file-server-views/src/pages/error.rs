use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{empty, home_button};
use crate::page::page;

/// Something went wrong, with a way back.
#[hotpath::measure]
pub fn error(page_title: &str, message: &str) -> impl IntoHtml {
    let bar = top_app_bar().headline(page_title).navigation(home_button());
    page()
        .title(page_title)
        .content(scaffold().top_app_bar(bar).body(empty(message)))
}
