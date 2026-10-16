//! Components: reusable views with their own builder API.

use crate::IntoHtml;

/// A view with its own builder API, rendered by building ordinary nodes.
///
/// Implementing this instead of [`IntoHtml`] means never naming the (long) type of the
/// element tree:
///
/// ```
/// use htmlb::prelude::*;
/// use htmlb::Component;
///
/// struct Chip<'a> { label: &'a str, selected: bool }
///
/// impl Component for Chip<'_> {
///     fn render(self) -> impl IntoHtml {
///         span().class(("chip", self.selected.then_some("selected"))).child(self.label)
///     }
///     fn dynamic_len_hint(&self) -> usize {
///         self.label.len()
///     }
/// }
///
/// let chip = Chip { label: "Rust", selected: true };
/// // The markup is known from the type alone; string contents (even `"chip"`) aren't.
/// assert_eq!(<Chip as IntoHtml>::MIN_LEN, r#"<span class=""></span>"#.len());
/// assert_eq!(chip.len_hint(), r#"<span class="">Rust</span>"#.len());
/// assert_eq!(div().child(chip).to_html(), r#"<div><span class="chip selected">Rust</span></div>"#);
/// ```
pub trait Component: Sized {
    fn render(self) -> impl IntoHtml;

    /// Cheap lower bound on the bytes `render`'s output writes beyond its
    /// [`MIN_LEN`](IntoHtml::MIN_LEN): text and other runtime-sized parts. Defaults to 0;
    /// override it so pages containing the component are sized precisely.
    #[inline]
    fn dynamic_len_hint(&self) -> usize {
        0
    }
}

/// `MIN_LEN` of the type a function returns. The type `Component::render` returns can't be
/// named on stable Rust, but it can be inferred from the function itself.
const fn min_len_of<S, H: IntoHtml>(_render: fn(S) -> H) -> usize {
    H::MIN_LEN
}

impl<T: Component> IntoHtml for T {
    const MIN_LEN: usize = min_len_of(T::render);
    #[inline]
    fn len_hint(&self) -> usize {
        Self::MIN_LEN + self.dynamic_len_hint()
    }
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
    #[inline]
    fn dynamic_len_hint(&self) -> usize {
        T::len_hint(self) - T::MIN_LEN
    }
}
