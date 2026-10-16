use htmlb::prelude::*;
use htmlb::{color_scheme, stylesheet, viewport};

pub fn document(
    page_title: impl IntoHtml,
    stylesheet_href: String,
    content: impl IntoHtml,
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
