use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::icons::Icons;
use crate::page::page;
use crate::urls::LoginHref;

/// For visitors who aren't signed in.
#[hotpath::measure]
pub fn signed_out(next: &str) -> impl IntoHtml {
    let body = (
        headline("Your files, on your server"),
        supporting_text("Sign in to browse, download, rename and move them."),
        filled_button("Sign in")
            .icon(Icons::Login)
            .href(LoginHref(next)),
    );
    page()
        .title("Sign in")
        .content(scaffold(top_app_bar("Files"), pane(body)))
}
