//! Material Design 3 for [`htmlb`]: the design system's tokens, components
//! configured only through them, and their stylesheet.

pub mod action;
pub mod color;
mod components;
pub mod css;
pub mod icon;
mod theme;
pub mod typography;

pub use action::{Action, Actionable};
pub use color::{Color, ColorRole, Scheme};
pub use components::*;
pub use theme::Theme;
pub use typography::{Typeface, Typefaces};

pub mod prelude {
    pub use crate::components::*;
    pub use crate::{Actionable, ColorRole, Theme, icon};
}
