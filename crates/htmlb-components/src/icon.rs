//! Material Symbols (Rounded, weight 400), drawn by Google Fonts' icon font. Each crate
//! declares the icons it uses with [`icons!`](crate::icons), getting an enum of them and
//! the URL of a stylesheet whose font holds only those icons.
//!
//! ```
//! htmlb_components::icons! {
//!     pub enum Icons { ArrowBack, Folder }
//! }
//!
//! use htmlb::prelude::IntoHtml;
//! assert_eq!(
//!     Icons::url(),
//!     "https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@24,400,0,0&icon_names=arrow_back,folder&display=block",
//! );
//! assert_eq!(Icons::Folder.to_html(), r#"<span class="md-icon" aria-hidden="true">folder</span>"#);
//! ```
//!
//! Unknown names don't compile:
//!
//! ```compile_fail
//! htmlb_components::icons! {
//!     pub enum Icons { NoSuchIcon }
//! }
//! let _ = Icons::url();
//! ```

mod names;

use htmlb::prelude::IntoHtml;

/// An icon. Rendered `aria-hidden`: pair it with a text label or `aria-label`.
pub trait Icon: IntoHtml + Copy {
    /// The ligature the font draws as this icon, e.g. `arrow_back`.
    fn name(self) -> &'static str;
}

/// Declares `enum Name { Variant, ... }` of Material Symbols, named in `PascalCase`
/// (`ArrowBack` is `arrow_back`). The enum is an [`Icon`], and `Name::url()` is the
/// stylesheet to link to on pages that use it.
#[macro_export]
macro_rules! icons {
    ($(#[$meta:meta])* $vis:vis enum $enum:ident { $($name:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum {
            $($name,)+
        }

        impl $enum {
            /// The Google Fonts stylesheet whose font draws exactly these icons.
            pub const fn url() -> &'static str {
            const NAMES: &[&str] = &[$($crate::__private::lookup(stringify!($name)),)+];
                const COUNT: usize = NAMES.len();
            const SORTED: [&str; COUNT] = $crate::__private::sorted::<COUNT>(NAMES);
            const LEN: usize = $crate::__private::url_len(&SORTED);
            const BYTES: [u8; LEN] = $crate::__private::url::<LEN>(&SORTED);
                match ::core::str::from_utf8(&BYTES) {
                    Ok(url) => url,
                    Err(_) => panic!("icon names are ASCII"),
                }
            }
        }

        impl $crate::__private::Icon for $enum {
            fn name(self) -> &'static str {
                match self {
                    $(Self::$name => const { $crate::__private::lookup(stringify!($name)) },)+
                }
            }
        }

        impl $crate::__private::CompileTimeHtmlSizeHint for $enum {
            const SIZE_HINT: usize = $crate::__private::MARKUP_LEN;
        }
        impl $crate::__private::RunTimeHtmlSizeHint for $enum {
            #[inline]
            fn size_hint(&self) -> usize {
                $crate::__private::Icon::name(*self).len()
            }
        }
        impl $crate::__private::IntoHtml for $enum {
            #[inline]
            fn write_html(self, buf: &mut String) {
                $crate::__private::write($crate::__private::Icon::name(self), buf)
            }
        }
    };
}

/// For [`icons!`](crate::icons) only.
#[doc(hidden)]
pub mod __private {
    pub use super::names::lookup;

    const PREFIX: &str = "https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@24,400,0,0&icon_names=";
    // `block` hides each ligature's text until the font arrives, rather than flashing it.
    const SUFFIX: &str = "&display=block";

    const OPEN: &str = r#"<span class="md-icon" aria-hidden="true">"#;
    const CLOSE: &str = "</span>";
    pub const MARKUP_LEN: usize = OPEN.len() + CLOSE.len();

    /// Writes an icon. Names are `[a-z0-9_]`: nothing to escape.
    #[inline]
    pub fn write(name: &str, buf: &mut String) {
        buf.push_str(OPEN);
        buf.push_str(name);
        buf.push_str(CLOSE);
    }

    /// Alphabetical, so the same icons give the same URL (and cache entry) in any order.
    pub const fn sorted<const N: usize>(names: &[&'static str]) -> [&'static str; N] {
        let mut out = [""; N];
        let mut i = 0;
        while i < N {
            out[i] = names[i];
            let mut j = i;
            while j > 0 && less(out[j], out[j - 1]) {
                let swap = out[j];
                out[j] = out[j - 1];
                out[j - 1] = swap;
                j -= 1;
            }
            i += 1;
        }
        out
    }

    const fn less(a: &str, b: &str) -> bool {
        let (a, b) = (a.as_bytes(), b.as_bytes());
        let mut i = 0;
        while i < a.len() && i < b.len() {
            if a[i] != b[i] {
                return a[i] < b[i];
            }
            i += 1;
        }
        a.len() < b.len()
    }

    /// The length of [`url`]: the names, comma-separated, between the prefix and suffix.
    pub const fn url_len(names: &[&str]) -> usize {
        let mut len = PREFIX.len() + SUFFIX.len() + names.len().saturating_sub(1);
        let mut i = 0;
        while i < names.len() {
            len += names[i].len();
            i += 1;
        }
        len
    }

    pub const fn url<const LEN: usize>(names: &[&str]) -> [u8; LEN] {
        let mut out = [0; LEN];
        let mut pos = copy(PREFIX, &mut out, 0);
        let mut i = 0;
        while i < names.len() {
            if i > 0 {
                out[pos] = b',';
                pos += 1;
            }
            pos = copy(names[i], &mut out, pos);
            i += 1;
        }
        copy(SUFFIX, &mut out, pos);
        out
    }

    const fn copy(s: &str, out: &mut [u8], mut pos: usize) -> usize {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            out[pos] = bytes[i];
            pos += 1;
            i += 1;
        }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::Icon;
    use htmlb::prelude::{IntoHtml, RunTimeHtmlSizeHint};

    crate::icons! {
        enum Icons { MoreVert, ArrowBack, Draft }
    }

    #[test]
    fn url_lists_names_alphabetically() {
        assert!(
            Icons::url().ends_with("&icon_names=arrow_back,draft,more_vert&display=block"),
            "{}",
            Icons::url()
        );
    }

    #[test]
    fn renders_a_hidden_ligature() {
        assert_eq!(Icons::MoreVert.name(), "more_vert");
        assert_eq!(
            Icons::MoreVert.to_html(),
            r#"<span class="md-icon" aria-hidden="true">more_vert</span>"#
        );
    }

    #[test]
    fn length_is_exact() {
        for icon in [Icons::ArrowBack, Icons::Draft, Icons::MoreVert] {
            assert_eq!(
                <Icons as htmlb::prelude::CompileTimeHtmlSizeHint>::SIZE_HINT + icon.size_hint(),
                icon.to_html().len()
            );
        }
    }
}
