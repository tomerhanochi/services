use crate::prelude::Unset;
use htmlb::prelude::*;

const BOTTOM_APP_BAR: &str = "md-bottom-app-bar";
const SELECTION_BAR: &str = "md-bottom-app-bar md-selection-bar";

/// A bar pinned to the bottom of the screen, within thumb reach, for a screen's main actions.
pub fn bottom_app_bar() -> BottomAppBar<Unset> {
    BottomAppBar {
        content: Unset,
        kind: BottomAppBarKind::Normal,
    }
}

/// A [`bottom_app_bar`] shown only while a [`checkbox`](crate::checkbox) on the page is
/// checked: for actions on the selection.
pub fn selection_bar() -> BottomAppBar<Unset> {
    BottomAppBar {
        content: Unset,
        kind: BottomAppBarKind::Selection,
    }
}

#[derive(Clone, Copy)]
enum BottomAppBarKind {
    Normal,
    Selection,
}

pub struct BottomAppBar<C> {
    content: C,
    kind: BottomAppBarKind,
}

impl<C> BottomAppBar<C> {
    pub fn child<N: IntoHtml>(self, child: N) -> BottomAppBar<N> {
        BottomAppBar {
            content: child,
            kind: self.kind,
        }
    }
}

impl<C: IntoHtml> Component for BottomAppBar<C> {
    fn render(self) -> impl IntoHtml {
        let class = match self.kind {
            BottomAppBarKind::Normal => BOTTOM_APP_BAR,
            BottomAppBarKind::Selection => SELECTION_BAR,
        };
        footer().class(class).child(self.content)
    }
}

impl<C: IntoHtml> RunTimeHtmlSizeHint for BottomAppBar<C> {
    fn size_hint(&self) -> usize {
        let class_len = match self.kind {
            BottomAppBarKind::Normal => BOTTOM_APP_BAR.len(),
            BottomAppBarKind::Selection => SELECTION_BAR.len(),
        };
        class_len + self.content.size_hint()
    }
}
