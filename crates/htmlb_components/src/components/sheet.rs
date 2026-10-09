use htmlb::prelude::*;
use htmlb::{AttrValue, Component};

use crate::class::{SHEET, SHEET_HANDLE, SHEET_HEADLINE};

/// A bottom sheet: a popover sliding up from the bottom edge on phones, centered on wider
/// screens. Open it with an action's [`opens`](crate::Actionable::opens) and the same `id`.
/// Tapping outside or pressing Escape closes it, without JavaScript.
pub fn sheet<I: AttrValue, H: IntoHtml>(id: I, headline: H) -> Sheet<I, H, ()> {
    Sheet {
        id,
        headline,
        content: (),
    }
}

pub struct Sheet<I, H, C> {
    id: I,
    headline: H,
    content: C,
}

impl<I, H, C> Sheet<I, H, C> {
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
    fn dynamic_len_hint(&self) -> usize {
        self.id.value_len_hint() + self.headline.len_hint() + self.content.len_hint()
    }
}
