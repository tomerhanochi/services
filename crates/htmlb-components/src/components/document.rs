use htmlb::prelude::*;
use htmlb::{Component, RunTimeHtmlSizeHint, color_scheme, viewport};

/// A whole page: the doctype, `<html lang="en">`, and a `<head>` that already declares the
/// charset, viewport and color schemes. Give it the rest of the [`head`](Document::head)
/// (title, stylesheets, ...) and a [`body`](Document::body).
pub fn document() -> Document<(), ()> {
    Document { head: (), body: () }
}

pub struct Document<H, B> {
    head: H,
    body: B,
}

impl<H, B> Document<H, B> {
    /// Appended to the `<head>`.
    pub fn head<H2: IntoHtml>(self, head: H2) -> Document<H2, B> {
        Document {
            head,
            body: self.body,
        }
    }

    /// The `<body>`'s contents.
    pub fn body<B2: IntoHtml>(self, body: B2) -> Document<H, B2> {
        Document {
            head: self.head,
            body,
        }
    }
}

impl<H: IntoHtml, B: IntoHtml> Component for Document<H, B> {
    fn render(self) -> impl IntoHtml {
        (
            doctype(),
            html().lang("en").child((
                head().child((
                    meta().charset("utf-8"),
                    viewport(),
                    color_scheme(),
                    self.head,
                )),
                body().child(self.body),
            )),
        )
    }
}
impl<H: IntoHtml, B: IntoHtml> RunTimeHtmlSizeHint for Document<H, B> {
    fn size_hint(&self) -> usize {
        self.head.size_hint() + self.body.size_hint()
    }
}
