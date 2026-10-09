use htmlb::prelude::*;

const PANE: &str = "md-pane";

/// A block of content with MD3's margins (16dp on phones, 24dp from 600dp), laying its
/// children out in a column with consistent spacing.
pub fn pane<C: IntoHtml>(content: C) -> impl IntoHtml {
    div().class(PANE).child(content)
}
