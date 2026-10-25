use htmlb::prelude::*;
use htmlb::{Component, RunTimeHtmlSizeHint, favicon, stylesheet};
use htmlb_components::Unset;
use htmlb_components::document;

use crate::icons::Icons;
use crate::urls::Asset;

/// A whole page: its [`content`](Page::content) under the head every page shares, with its
/// [`title`](Page::title) before the site's name.
pub fn page() -> Page<Unset, Unset> {
    Page {
        title: Unset,
        content: Unset,
    }
}

pub struct Page<T, C> {
    title: T,
    content: C,
}

impl<T, C> Page<T, C> {
    /// The tab's title, before ` | File Server`.
    pub fn title<T2: IntoHtml>(self, title: T2) -> Page<T2, C> {
        Page {
            title,
            content: self.content,
        }
    }

    /// The `<body>`'s contents, usually a [`scaffold`](htmlb_components::scaffold).
    pub fn content<C2: IntoHtml>(self, content: C2) -> Page<T, C2> {
        Page {
            title: self.title,
            content,
        }
    }
}

impl<T: IntoHtml, C: IntoHtml> Component for Page<T, C> {
    fn render(self) -> impl IntoHtml {
        document()
            .head((
                title().child((self.title, " | File Server")),
                stylesheet().href(Asset::Css.url()),
                stylesheet().href(Icons::url()),
                favicon().href(Asset::Favicon.url()),
            ))
            .body(self.content)
    }
}
impl<T: IntoHtml, C: IntoHtml> RunTimeHtmlSizeHint for Page<T, C> {
    fn size_hint(&self) -> usize {
        self.title.size_hint() + self.content.size_hint()
    }
}
