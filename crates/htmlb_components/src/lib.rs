//! Material Design 3 for [`htmlb`]: the design system's tokens, components
//! configured only through them, and their stylesheet.
//!
//! # Tokens
//!
//! MD3's system tokens are the vocabulary components speak. A component never takes
//! `#6750a4` or `14px`; it takes [`ColorRole::Primary`] or [`TypeScale::LabelLarge`], and
//! the [`Theme`] decides what those are.
//!
//! - [`color`]: color roles, and the [`Scheme`]s that give them values.
//! - [`typography`]: the type scale and the typefaces behind it.
//! - [`shape`]: the corner-radius scale.
//! - [`elevation`], [`state`], [`motion`]: shadows, interaction overlays, and animation.
//!
//! Fixed values are MD3's, from `@material/web` 2.5.0's token files
//! (`tokens/versions/v0_192`). Colors and typefaces come from a [`Theme`], which this
//! crate doesn't provide: `htmlb_components_baseline` has MD3's baseline.
//! [`css::css`] turns a theme into a complete stylesheet with CSS custom properties.
//!
//! # Components
//!
//! There is no way to pass a component a class, a style, a hex color or a pixel size, so
//! pages stay consistent with the system. Components render markup with the class names in
//! [`class`], which [`CSS`] styles. Generate a complete themed stylesheet with
//! [`css::css`], and pass its URL to [`document`].
//!
//! ```
//! use htmlb::prelude::*;
//! use htmlb_components::prelude::*;
//!
//! let row = list_item("Photos").leading(icon::Folder).supporting("12 items").href("/m/photos/");
//! let html = list(row).to_html();
//! assert!(html.starts_with(r#"<ul class="md-list"><li class="md-list-item"><a class="md-list-item__main" href="/m/photos/">"#));
//! ```
//!
//! Interactive components take an *action* through [`Actionable`]:
//! [`href`](Actionable::href) renders a link, [`submit`](Actionable::submit) a submit
//! button, [`opens`](Actionable::opens) a button showing a popover ([`sheet`]). Nothing
//! needs JavaScript.
//!
//! Icons are types in [`icon`], e.g. [`icon::ArrowBack`].
//!
//! Constructors are named so both preludes can be imported at once: `filled_button`, not
//! `button` (which is `htmlb`'s `<button>`).

pub mod action;
pub mod class;
pub mod color;
mod components;
pub mod css;
pub mod elevation;
pub mod icon;
pub mod motion;
pub mod shape;
pub mod state;
mod stylesheet;
mod theme;
pub mod typography;

pub use action::{Action, Actionable};
pub use color::{Color, ColorRole, Scheme};
pub use components::*;
pub use elevation::Elevation;
pub use motion::{Duration, Easing};
pub use shape::{Corners, Shape};
pub use state::StateLayer;
pub use stylesheet::CSS;
pub use theme::Theme;
pub use typography::{TypeScale, TypeStyle, Typeface, Typefaces};

pub mod prelude {
    pub use crate::components::*;
    pub use crate::{Actionable, ColorRole, Elevation, Shape, Theme, TypeScale, icon};
}
