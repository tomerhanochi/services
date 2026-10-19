//! Common document components with consistent metadata and resource attributes.

use crate::{IntoHtml, prelude::*};

/// The standard responsive viewport declaration, including safe-area support.
pub fn viewport() -> impl IntoHtml {
    meta()
        .name("viewport")
        .content("width=device-width, initial-scale=1, viewport-fit=cover")
}

/// Declares support for both light and dark browser UI controls.
pub fn color_scheme() -> impl IntoHtml {
    meta().name("color-scheme").content("light dark")
}

/// `<link rel="…">`, before its `href`.
type Link = crate::El<crate::tag::link, ((), crate::Attr<crate::attr::rel, &'static str>), ()>;

/// A stylesheet link: give it an `href`.
pub fn stylesheet() -> Link {
    link().rel("stylesheet")
}

/// A favicon link: give it an `href`.
pub fn favicon() -> Link {
    link().rel("icon")
}

/// An external script with a Subresource Integrity SHA tag.
pub fn script(src: impl AsRef<str>, sha: impl AsRef<str>) -> impl IntoHtml {
    script_element()
        .src(src.as_ref().to_owned())
        .integrity(sha.as_ref().to_owned())
        .crossorigin(CrossOrigin::Anonymous)
        .defer(true)
}

use crate::attrs::enums::CrossOrigin;
use crate::elements::constructors::script as script_element;
