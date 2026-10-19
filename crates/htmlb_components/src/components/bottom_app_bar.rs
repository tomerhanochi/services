use htmlb::prelude::*;

const BOTTOM_APP_BAR: &str = "md-bottom-app-bar";
const SELECTION_BAR: &str = "md-bottom-app-bar md-selection-bar";

/// A bar pinned to the bottom of the screen, within thumb reach, for a screen's main actions.
pub fn bottom_app_bar<C: IntoHtml>(content: C) -> impl IntoHtml {
    footer().class(BOTTOM_APP_BAR).child(content)
}

/// A [`bottom_app_bar`] shown only while a [`checkbox`](crate::checkbox) on the page is
/// checked: for actions on the selection.
pub fn selection_bar<C: IntoHtml>(content: C) -> impl IntoHtml {
    footer().class(SELECTION_BAR).child(content)
}
