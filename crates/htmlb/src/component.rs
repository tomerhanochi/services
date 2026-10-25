//! Components: reusable views with their own builder API.

use crate::into_html::IntoHtml;
use crate::size_hint::{CompileTimeHtmlSizeHint, RunTimeHtmlSizeHint};

/// A view with its own builder API, rendered by building ordinary nodes.
///
/// Implementing this instead of [`IntoHtml`] means never naming the (long) type of the
/// element tree:
///
/// ```
/// use htmlb::prelude::*;
///
/// struct Chip<'a> { label: &'a str, selected: bool }
///
/// impl Component for Chip<'_> {
///     fn render(self) -> impl IntoHtml {
///         span().class(("chip", self.selected.then_some("selected"))).child(self.label)
///     }
/// }
/// impl RunTimeHtmlSizeHint for Chip<'_> {
///     fn size_hint(&self) -> usize {
///         self.label.len()
///     }
/// }
///
/// let chip = Chip { label: "Rust", selected: true };
/// // The baseline is known from the type; runtime additions cover value-dependent output.
/// assert_eq!(<Chip as CompileTimeHtmlSizeHint>::SIZE_HINT, r#"<span class=""></span>"#.len());
/// assert_eq!(Chip::SIZE_HINT + chip.size_hint(), r#"<span class="">Rust</span>"#.len());
/// assert_eq!(div().child(chip).to_html(), r#"<div><span class="chip selected">Rust</span></div>"#);
/// ```
pub trait Component: Sized + RunTimeHtmlSizeHint {
    fn render(self) -> impl IntoHtml;
}

/// `SIZE_HINT` of the type a function returns. The type `Component::render` returns can't be
/// named on stable Rust, but it can be inferred from the function itself.
const fn min_size_of<S, H: IntoHtml>(_render: fn(S) -> H) -> usize {
    H::SIZE_HINT
}

impl<T: Component> CompileTimeHtmlSizeHint for T {
    const SIZE_HINT: usize = min_size_of(T::render);
}

impl<T: Component> IntoHtml for T {
    #[inline]
    fn write_html(self, buf: &mut String) {
        self.render().write_html(buf)
    }
}

/// `Box<T>` renders as `T`. (It's a component because `Box` is a fundamental type: a
/// direct `IntoHtml` impl would overlap with the blanket impl above.)
impl<T: IntoHtml> Component for Box<T> {
    #[inline]
    fn render(self) -> impl IntoHtml {
        *self
    }
}

impl<T: IntoHtml> RunTimeHtmlSizeHint for Box<T> {
    #[inline]
    fn size_hint(&self) -> usize {
        T::size_hint(self)
    }
}
