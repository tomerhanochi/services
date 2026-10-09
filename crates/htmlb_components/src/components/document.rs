use htmlb::prelude::*;
use htmlb::{color_scheme, stylesheet, viewport};

/// A whole HTML document. `stylesheets` are URLs for the app's generated theme CSS and
/// any other stylesheets. The theme also colors the browser's own UI to match.
pub fn document<B: IntoHtml>(
    page_title: String,
    stylesheet_href: String,
    content: B,
) -> impl IntoHtml {
    (
        doctype(),
        html().lang("en").child((
            head().child((
                meta().charset("utf-8"),
                viewport(),
                color_scheme(),
                title().child(page_title),
                stylesheet(stylesheet_href),
            )),
            body().child(content),
        )),
    )
}
