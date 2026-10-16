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
//! Every node reports how many bytes it will write:
//!
//! - [`IntoHtml::MIN_LEN`]: a compile-time constant computed from the *type* alone:
//!   tag names, `<`/`>`/`</…>`, attribute names, `="`, and nested constants.
//! - [`IntoHtml::len_hint`]: a cheap runtime lower bound: adds text lengths (unescaped)
//!   and number widths. Never larger than the output.
//!
//! [`IntoHtml::to_html`] reserves `len_hint()` plus 1/8 headroom for escape expansion, so
//! a page renders with one allocation. Lists made with
//! [`into_html_cloned`](IteratorExt::into_html_cloned) contribute their full length;
//! [`into_html`](IteratorExt::into_html) lists only `count × MIN_LEN`.
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
//! let _ = script().child(div()); // <script> holds raw text only
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
mod num;
mod text;

pub use attrs::{
    Attr, AttrKey, AttrValue, Attribute, BoolAttr, DynAttr, Typed, enums::*, keys as attr,
};
pub use class::{ClassList, Classes};
pub use component::Component;
pub use components::{color_scheme, script, stylesheet, viewport};
pub use containers::{ClonedIterHtml, Either, IterHtml, IteratorExt};
pub use elements::{Child, El, HasChildren, Kind, Tag, Unescaped, kind, tag, *};
pub use text::{Display, Doctype, Raw, RawStr, display, doctype, raw};

pub mod prelude {
    pub use crate::{
        Either, IntoHtml, IteratorExt, attrs::enums::*, display, doctype,
        elements::constructors::*, raw,
    };
}

/// Anything that can be rendered as HTML: elements, text, numbers, and containers of them.
#[diagnostic::on_unimplemented(message = "`{Self}` can't be rendered as HTML")]
pub trait IntoHtml: Sized {
    /// Bytes this value always writes, known from its type alone.
    const MIN_LEN: usize;

    /// Cheap lower bound on the bytes [`write_html`](Self::write_html) will write:
    /// exact for markup and numbers, unescaped length for text.
    #[inline]
    fn len_hint(&self) -> usize {
        Self::MIN_LEN
    }

    /// Appends the HTML to `buf`. Does not reserve.
    fn write_html(self, buf: &mut String);

    /// Reserves [`len_hint`](Self::len_hint) plus 1/8 headroom for escape expansion, then
    /// renders. To reuse a buffer across renders, call [`write_html`](Self::write_html).
    fn to_html(self) -> String {
        let hint = self.len_hint();
        let mut buf = String::with_capacity(hint + hint / 8);
        self.write_html(&mut buf);
        buf
    }
}
