//! Theme-selected typefaces (`md.ref.typeface.*`). Fixed text styles live in CSS.

/// Which of the theme's [`Typefaces`] a style uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Typeface {
    /// For large text: display, headline, title-large.
    Brand,
    /// For everything else.
    Plain,
}

/// The two font families a theme provides. Each is a CSS-style family list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Typefaces {
    pub brand: &'static str,
    pub plain: &'static str,
}

impl Typefaces {
    pub const fn get(&self, typeface: Typeface) -> &'static str {
        match typeface {
            Typeface::Brand => self.brand,
            Typeface::Plain => self.plain,
        }
    }
}
