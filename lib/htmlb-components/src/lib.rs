//! Material Design 3 for [`htmlb`]: the design system's tokens, components
//! configured only through them, and their stylesheet.

mod action;
mod color;
mod components;
mod css;
mod icon;
mod theme;
mod typography;
mod unset;

pub mod prelude {
    pub use crate::action::{
        Action, Actionable, Download, HidePopover, Inert, Label, Link, ShowPopover, Submit,
    };
    pub use crate::color::{Color, ColorRole, Scheme};
    pub use crate::components::*;
    pub use crate::css::css;
    pub use crate::icon::Icon;
    pub use crate::icons;
    pub use crate::theme::Theme;
    pub use crate::typography::{Typeface, Typefaces};
    pub use crate::unset::Unset;
}

/// Implementation details used by the exported `icons!` macro.
#[doc(hidden)]
pub mod __private {
    pub use crate::icon::__private::{MARKUP_LEN, lookup, sorted, url, url_len, write};
    pub use crate::icon::Icon;
    pub use htmlb::prelude::{CompileTimeHtmlSizeHint, IntoHtml, RunTimeHtmlSizeHint};
}
