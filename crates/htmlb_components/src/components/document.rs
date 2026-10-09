use htmlb::prelude::*;
use htmlb::{color_scheme, stylesheet, viewport};

use crate::{Scheme, Theme};

/// A whole HTML document. `stylesheets` are URLs for the app's generated theme CSS and
/// any other stylesheets. The theme also colors the browser's own UI to match.
pub fn document<T: IntoHtml, B: IntoHtml>(
    theme: &Theme,
    stylesheets: &[&str],
    page_title: T,
    content: B,
) -> impl IntoHtml + use<T, B> {
    let theme_color = |scheme: &Scheme, media| {
        meta()
            .name("theme-color")
            .content(scheme.surface)
            .attr("media", media)
    };
    (
        doctype(),
        html().lang("en").child((
            head().child((
                meta().charset("utf-8"),
                viewport(),
                color_scheme(),
                theme_color(&theme.light, "(prefers-color-scheme: light)"),
                theme_color(&theme.dark, "(prefers-color-scheme: dark)"),
                title().child(page_title),
                stylesheets
                    .iter()
                    .map(|&href| stylesheet(href.to_owned()))
                    .collect::<Vec<_>>(),
            )),
            body().child(content),
        )),
    )
}
