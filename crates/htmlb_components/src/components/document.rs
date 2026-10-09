use htmlb::prelude::*;

use crate::{Scheme, Theme};

/// A whole HTML document. `stylesheets` are the URLs of the theme's CSS ([`Theme::css`])
/// and the components' ([`CSS`](crate::CSS)); `theme` also colors the browser's own UI to
/// match.
pub fn document<'a, T: IntoHtml + 'a, B: IntoHtml + 'a>(
    theme: &'a Theme,
    stylesheets: &'a [&'a str],
    page_title: T,
    content: B,
) -> impl IntoHtml + 'a {
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
                meta()
                    .name("viewport")
                    .content("width=device-width, initial-scale=1, viewport-fit=cover"),
                meta().name("color-scheme").content("light dark"),
                theme_color(&theme.light, "(prefers-color-scheme: light)"),
                theme_color(&theme.dark, "(prefers-color-scheme: dark)"),
                title().child(page_title),
                stylesheets
                    .iter()
                    .map(|&href| link().rel("stylesheet").href(href))
                    .into_html_cloned(),
            )),
            body().child(content),
        )),
    )
}
