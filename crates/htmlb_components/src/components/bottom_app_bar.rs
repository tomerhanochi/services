use htmlb::prelude::*;

const BOTTOM_APP_BAR: &str = "md-bottom-app-bar";

/// A bar pinned to the bottom of the screen, within thumb reach, for a screen's main actions.
pub fn bottom_app_bar<C: IntoHtml>(content: C) -> impl IntoHtml {
    footer().class(BOTTOM_APP_BAR).child(content)
}
