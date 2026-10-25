use crate::prelude::Unset;
use htmlb::prelude::*;

const PANE: &str = "md-pane";

/// A block of content with MD3's margins (16dp on phones, 24dp from 600dp), laying its
/// children out in a column with consistent spacing.
pub fn pane() -> Pane<Unset> {
    Pane { content: Unset }
}

pub struct Pane<C> {
    content: C,
}

impl<C> Pane<C> {
    pub fn child<N: IntoHtml>(self, child: N) -> Pane<N> {
        Pane { content: child }
    }
}

impl<C: IntoHtml> Component for Pane<C> {
    fn render(self) -> impl IntoHtml {
        div().class(PANE).child(self.content)
    }
}

impl<C: IntoHtml> RunTimeHtmlSizeHint for Pane<C> {
    fn size_hint(&self) -> usize {
        PANE.len() + self.content.size_hint()
    }
}
