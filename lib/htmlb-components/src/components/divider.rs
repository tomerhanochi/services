use htmlb::prelude::*;

const DIVIDER: &str = "md-divider";

/// A horizontal rule between groups of content.
pub fn divider() -> impl IntoHtml {
    hr().class(DIVIDER)
}
