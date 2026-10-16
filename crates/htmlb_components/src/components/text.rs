use htmlb::{IntoHtml, h1, h2, h3, p, span};

/// The primary page heading: an `<h1>` using Material's display-large style.
pub fn display<C: IntoHtml>(content: C) -> impl IntoHtml {
    h1().class("md-type-display-large").child(content)
}

/// A section heading: an `<h2>` using Material's headline-large style.
pub fn headline<C: IntoHtml>(content: C) -> impl IntoHtml {
    h2().class("md-type-headline-large").child(content)
}

/// A subsection heading: an `<h3>` using Material's title-large style.
pub fn title<C: IntoHtml>(content: C) -> impl IntoHtml {
    h3().class("md-type-title-large").child(content)
}

/// A supporting line below a heading, styled as body-medium secondary text.
pub fn subtitle<C: IntoHtml>(content: C) -> impl IntoHtml {
    p().class(("md-type-body-medium", "md-color-on-surface-variant"))
        .child(content)
}

/// A prose paragraph using Material's body-large style.
pub fn paragraph<C: IntoHtml>(content: C) -> impl IntoHtml {
    p().class("md-type-body-large").child(content)
}

/// Secondary explanatory text using body-small and the on-surface-variant color.
pub fn supporting_text<C: IntoHtml>(content: C) -> impl IntoHtml {
    p().class(("md-type-body-small", "md-color-on-surface-variant"))
        .child(content)
}

/// Concise label text using Material's label-large style.
pub fn label<C: IntoHtml>(content: C) -> impl IntoHtml {
    span().class("md-type-label-large").child(content)
}
