//! A zero-dependency, type-checked HTML builder for server-side rendering.
//!
//! ```
//! use htmlb::prelude::*;
//!
//! let teams = ["Jiangsu", "Beijing"];
//! let page = ul().class("teams").child(teams.iter().map(|t| li().child(*t)).into_html());
//! assert_eq!(page.to_html(), r#"<ul class="teams"><li>Jiangsu</li><li>Beijing</li></ul>"#);
//! ```
//!
//! Helpers return `impl IntoHtml`, whatever element they build:
//!
//! ```
//! use htmlb::prelude::*;
//!
//! fn row(name: &str, score: u8) -> impl IntoHtml + '_ {
//!     tr().child((td().child(name), td().child(score)))
//! }
//!
//! let html = table().child((row("Jiangsu", 43), row("Beijing", 27))).to_html();
//! assert_eq!(html, "<table><tr><td>Jiangsu</td><td>43</td></tr><tr><td>Beijing</td><td>27</td></tr></table>");
//! ```
//!
//! # Size estimates
//!
//! Every node has a compile-time baseline and can report an additional runtime estimate:
//!
//! - [`CompileTimeHtmlSizeHint::SIZE_HINT`]: a compile-time constant computed from the *type* alone:
//!   tag names, `<`/`>`/`</…>`, attribute names, `="`, and nested constants.
//! - [`RunTimeHtmlSizeHint::size_hint`]: bytes to add to that baseline for value-dependent output,
//!   such as text lengths (unescaped) and number widths.
//!
//! [`IntoHtml::to_html`] reserves `SIZE_HINT + size_hint()` plus 1/8 headroom for escape
//! expansion. Lists made with [`into_html_cloned`](IteratorExt::into_html_cloned) contribute
//! their full length; [`into_html`](IteratorExt::into_html) lists only `count × SIZE_HINT`.
//!
//! There is deliberately no exact length: counting escapes means scanning every string
//! twice, which benchmarks showed costs more than the reallocation it avoids.
//!
//! Strings that can't contain `& < > " '` (percent-encoded URLs, formatted dates) can be
//! passed as [`raw`] to skip escaping.
//!
//! # Type safety
//!
//! - Attributes are per element (`a().href(..)` compiles, `div().href(..)` doesn't).
//! - Attribute values are typed: boolean attributes take `bool`, numeric ones take numbers,
//!   enumerated ones take enums (`input().r#type(InputType::Email)`).
//! - Void elements (`<img>`, `<br>`, ...) have no children. `<script>`/`<style>` take
//!   strings only, written unescaped; everything else escapes its text.
//! - Which element may contain which (e.g. `<li>` only in lists) is *not* checked, so that
//!   helpers can always return a plain `impl IntoHtml`.
//! - Custom attribute names must be `&'static str`, so they can't come from user input.
//!
//! ```compile_fail
//! use htmlb::prelude::*;
//! let _ = img().child("void elements have no children");
//! ```
//! ```compile_fail
//! use htmlb::prelude::*;
//! let _ = constructors::script().child(div()); // <script> holds raw text only
//! ```
//! ```compile_fail
//! use htmlb::prelude::*;
//! let _ = div().href("/"); // no such attribute on <div>
//! ```
//! ```compile_fail
//! use htmlb::prelude::*;
//! let _ = input().r#type("emial"); // enumerated attribute: use InputType
//! ```
//! ```compile_fail
//! use htmlb::prelude::*;
//! let _ = button().disabled("false"); // boolean attribute: takes bool
//! ```
//! ```compile_fail
//! use htmlb::prelude::*;
//! let name = String::from("onclick");
//! let _ = div().attr(&name, "x"); // attribute names must be &'static str
//! ```

mod attrs;
mod class;
mod component;
mod components;
mod containers;
mod elements;
mod escape;
mod into_html;
mod num;
mod size_hint;
mod text;

pub mod prelude {
    pub use crate::attrs::{
        Attr, AttrKey, AttrValue, Attribute, BoolAttr, DynAttr, Typed, enums::*, keys as attr,
    };
    pub use crate::class::{ClassList, Classes};
    pub use crate::component::Component;
    pub use crate::components::{color_scheme, favicon, script, stylesheet, viewport};
    pub use crate::containers::{ClonedIterHtml, Either, IterHtml, IteratorExt};
    pub use crate::elements::{Child, El, HasChildren, Kind, Tag, Unescaped, kind, tag, *};
    pub use crate::into_html::IntoHtml;
    pub use crate::size_hint::{CompileTimeHtmlSizeHint, RunTimeHtmlSizeHint};
    pub use crate::text::{Display, Doctype, Raw, RawStr, display, doctype, raw};
}
