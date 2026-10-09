//! Leaf content: strings, chars, numbers, raw HTML, doctype.
//!
//! Strings and chars are escaped. (`<script>`/`<style>` children bypass this; see
//! [`Unescaped`](crate::Unescaped).)

use std::borrow::Cow;
use std::fmt;

use crate::IntoHtml;
use crate::{escape, num};

macro_rules! strings {
    ($([$($gen:tt)*] $ty:ty => |$s:ident| $as_str:expr;)*) => {$(
        impl<$($gen)*> IntoHtml for $ty {
            const MIN_LEN: usize = 0;
            #[inline]
            fn len_hint(&self) -> usize {
                let $s = self;
                let s: &str = $as_str;
                s.len()
            }
            #[inline]
            fn write_html(self, buf: &mut String) {
                let $s = &self;
                escape::write(buf, $as_str)
            }
        }
    )*};
}

strings! {
    ['a] &'a str => |s| s;
    [] String => |s| s.as_str();
    ['a] &'a String => |s| s.as_str();
    ['a] Cow<'a, str> => |s| s;
    [] Box<str> => |s| s;
}

impl IntoHtml for char {
    const MIN_LEN: usize = 1;
    #[inline]
    fn len_hint(&self) -> usize {
        self.len_utf8()
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        escape::write(buf, self.encode_utf8(&mut [0; 4]))
    }
}

/// Numbers never need escaping. Integer `len_hint` is exact; floats and 128-bit
/// integers report 1 (`to_html`'s headroom absorbs the difference).
macro_rules! numbers {
    (@int $($ty:ty)* => $wide:ty, $digits:path, $write:path) => {$(
        impl IntoHtml for $ty {
            const MIN_LEN: usize = 1;
            #[inline]
            fn len_hint(&self) -> usize {
                $digits(*self as $wide)
            }
            #[inline]
            fn write_html(self, buf: &mut String) {
                $write(buf, self as $wide)
            }
        }
    )*};
    (@display $($ty:ty)*) => {$(
        impl IntoHtml for $ty {
            const MIN_LEN: usize = 1;
            #[inline]
            fn write_html(self, buf: &mut String) {
                num::write_display(buf, self)
            }
        }
    )*};
}

numbers!(@int u8 u16 u32 u64 usize => u64, num::digits, num::write_u64);
numbers!(@int i8 i16 i32 i64 isize => i64, num::digits_signed, num::write_i64);
numbers!(@display u128 i128 f32 f64);

/// Text or markup that is already safe, written verbatim with no escaping, as a child
/// or as an attribute value. Use it for pre-escaped strings, values that can't contain
/// `& < > "` (percent-encoded URLs, formatted dates, numbers), and static markup such as
/// SVG icons.
///
/// Never wrap untrusted input: it is not checked.
pub struct Raw<S>(pub S);

/// [`Raw`] over a borrowed string.
pub type RawStr<'a> = Raw<&'a str>;

pub fn raw<S: AsRef<str>>(html: S) -> Raw<S> {
    Raw(html)
}

impl<S: AsRef<str>> crate::AttrValue for Raw<S> {
    const VALUE_MIN_LEN: usize = 0;
    #[inline]
    fn value_len_hint(&self) -> usize {
        self.0.as_ref().len()
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        buf.push_str(self.0.as_ref())
    }
}

impl<S: AsRef<str>> IntoHtml for Raw<S> {
    const MIN_LEN: usize = 0;
    #[inline]
    fn len_hint(&self) -> usize {
        self.0.as_ref().len()
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        buf.push_str(self.0.as_ref())
    }
}

/// Any [`Display`](fmt::Display) value, formatted straight into the output and escaped,
/// as a child or as an attribute value. Saves building a `String` first for sizes, dates
/// and other formatted values.
///
/// Its size estimate is 0: the length isn't known until it is formatted.
pub struct Display<T>(pub T);

pub fn display<T: fmt::Display>(value: T) -> Display<T> {
    Display(value)
}

/// Escapes everything written through it into the buffer.
struct Escaping<'a>(&'a mut String);

impl fmt::Write for Escaping<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        escape::write(self.0, s);
        Ok(())
    }
}

impl<T: fmt::Display> IntoHtml for Display<T> {
    const MIN_LEN: usize = 0;
    #[inline]
    fn write_html(self, buf: &mut String) {
        use fmt::Write;
        let _ = write!(Escaping(buf), "{}", self.0);
    }
}

impl<T: fmt::Display> crate::AttrValue for Display<T> {
    const VALUE_MIN_LEN: usize = 0;
    #[inline]
    fn value_len_hint(&self) -> usize {
        0
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        use fmt::Write;
        let _ = write!(Escaping(buf), "{}", self.0);
    }
}

/// `<!DOCTYPE html>`
pub struct Doctype;

pub fn doctype() -> Doctype {
    Doctype
}

impl IntoHtml for Doctype {
    const MIN_LEN: usize = "<!DOCTYPE html>".len();
    #[inline]
    fn write_html(self, buf: &mut String) {
        buf.push_str("<!DOCTYPE html>")
    }
}
