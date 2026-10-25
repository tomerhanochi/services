use crate::prelude::Unset;
use htmlb::prelude::*;

const SHEET: &str = "md-sheet";
const SHEET_HANDLE: &str = "md-sheet__handle";
const SHEET_HEADLINE: &str = "md-sheet__headline";

/// A bottom sheet: a popover sliding up from the bottom edge on phones, centered on wider
/// screens. Open it with an action's [`opens`](crate::Actionable::opens) and the same `id`.
/// Tapping outside or pressing Escape closes it, without JavaScript.
pub fn sheet() -> Sheet<Unset, Unset, ()> {
    Sheet {
        id: None,
        headline: Unset,
        content: (),
    }
}

pub struct Sheet<I, H, C> {
    id: Option<I>,
    headline: H,
    content: C,
}

impl<I, H, C> Sheet<I, H, C> {
    pub fn id<I2: AttrValue>(self, id: I2) -> Sheet<I2, H, C> {
        Sheet {
            id: Some(id),
            headline: self.headline,
            content: self.content,
        }
    }

    pub fn headline<H2: IntoHtml>(self, headline: H2) -> Sheet<I, H2, C> {
        Sheet {
            id: self.id,
            headline,
            content: self.content,
        }
    }

    pub fn child<N: IntoHtml>(self, child: N) -> Sheet<I, H, (C, N)> {
        Sheet {
            id: self.id,
            headline: self.headline,
            content: (self.content, child),
        }
    }
}

impl<I: AttrValue, H: IntoHtml, C: IntoHtml> Component for Sheet<I, H, C> {
    fn render(self) -> impl IntoHtml {
        div()
            .id(self.id)
            .popover(Popover::Auto)
            .class(SHEET)
            .role("dialog")
            .child((
                div().class(SHEET_HANDLE).aria_hidden("true"),
                p().class(SHEET_HEADLINE).child(self.headline),
                self.content,
            ))
    }
}
impl<I: AttrValue, H: IntoHtml, C: IntoHtml> RunTimeHtmlSizeHint for Sheet<I, H, C> {
    fn size_hint(&self) -> usize {
        self.id.size_hint() + self.headline.size_hint() + self.content.size_hint()
    }
}
