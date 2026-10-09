use htmlb::prelude::*;

use crate::class::DIVIDER;

/// A horizontal rule between groups of content.
pub fn divider() -> impl IntoHtml {
    hr().class(DIVIDER)
}
