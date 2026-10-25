use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::icons::Icons;
use crate::page::page;
use crate::urls::LoginHref;

/// For visitors who aren't signed in.
#[hotpath::measure]
pub fn signed_out(next: &str) -> impl IntoHtml {
    let body = (
        headline().child("Your files, on your server"),
        supporting_text().child("Sign in to browse, download, rename and move them."),
        filled_button()
            .label("Sign in")
            .icon(Icons::Login)
            .href(LoginHref(next)),
    );
    page().title("Sign in").content(
        scaffold()
            .top_app_bar(top_app_bar().headline("Files"))
            .body(pane().child(body)),
    )
}
