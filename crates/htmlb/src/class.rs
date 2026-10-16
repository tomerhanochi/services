//! The `class` attribute's value: a list of class names, joined with spaces.

use std::borrow::Cow;

use crate::{AttrValue, escape};

/// One or more class names. Tuples are lists: `.class(("btn", active.then_some("active")))`
/// writes `btn active`, or `btn` when inactive. Empty and `None` parts are skipped, so no
/// stray spaces are written.
pub trait ClassList {
    /// False when every part may be `None`: then the whole attribute may be omitted.
    const ALWAYS_PRESENT: bool = true;
    #[inline]
    fn is_present(&self) -> bool {
        true
    }
    /// Cheap lower bound on the bytes written, counting a separating space before every
    /// non-empty class (including the first, which [`Classes`] subtracts).
    fn classes_len_hint(&self) -> usize;
    /// Writes each non-empty class, preceded by a space unless nothing has been written to
    /// `buf` since `start`.
    fn write_classes(self, buf: &mut String, start: usize);
}

/// A [`ClassList`] as an attribute value. Built by the `class` attribute method.
pub struct Classes<C>(pub(crate) C);

impl<C: ClassList> AttrValue for Classes<C> {
    const VALUE_MIN_LEN: usize = 0;
    const ALWAYS_PRESENT: bool = C::ALWAYS_PRESENT;
    #[inline]
    fn is_present(&self) -> bool {
        self.0.is_present()
    }
    #[inline]
    fn value_len_hint(&self) -> usize {
        self.0.classes_len_hint().saturating_sub(1)
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        let start = buf.len();
        self.0.write_classes(buf, start)
    }
}

#[inline]
fn write_class(buf: &mut String, start: usize, class: &str) {
    if class.is_empty() {
        return;
    }
    if buf.len() > start {
        buf.push(' ');
    }
    escape::write(buf, class)
}

#[inline]
fn class_len_hint(class: &str) -> usize {
    if class.is_empty() { 0 } else { class.len() + 1 }
}

macro_rules! string_classes {
    ($([$($gen:tt)*] $ty:ty => |$s:ident| $as_str:expr;)*) => {$(
        impl<$($gen)*> ClassList for $ty {
            #[inline]
            fn classes_len_hint(&self) -> usize {
                let $s = self;
                class_len_hint($as_str)
            }
            #[inline]
            fn write_classes(self, buf: &mut String, start: usize) {
                let $s = &self;
                write_class(buf, start, $as_str)
            }
        }
    )*};
}

string_classes! {
    ['a] &'a str => |s| s;
    [] String => |s| s.as_str();
    ['a] &'a String => |s| s.as_str();
    ['a] Cow<'a, str> => |s| s;
    [] Box<str> => |s| s;
}

/// `None` writes nothing; if every part is `None`, the attribute is omitted.
impl<C: ClassList> ClassList for Option<C> {
    const ALWAYS_PRESENT: bool = false;
    #[inline]
    fn is_present(&self) -> bool {
        self.as_ref().is_some_and(C::is_present)
    }
    #[inline]
    fn classes_len_hint(&self) -> usize {
        self.as_ref().map_or(0, C::classes_len_hint)
    }
    #[inline]
    fn write_classes(self, buf: &mut String, start: usize) {
        if let Some(c) = self {
            c.write_classes(buf, start)
        }
    }
}

macro_rules! tuple_classes {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: ClassList),+> ClassList for ($($t,)+) {
            const ALWAYS_PRESENT: bool = false $(|| $t::ALWAYS_PRESENT)+;
            #[inline]
            fn is_present(&self) -> bool {
                let ($($t,)+) = self;
                false $(|| $t.is_present())+
            }
            #[inline]
            fn classes_len_hint(&self) -> usize {
                let ($($t,)+) = self;
                0 $(+ $t.classes_len_hint())+
            }
            #[inline]
            fn write_classes(self, buf: &mut String, start: usize) {
                let ($($t,)+) = self;
                $($t.write_classes(buf, start);)+
            }
        }
    };
}

macro_rules! all_tuple_classes {
    ($first:ident $($rest:ident)*) => {
        tuple_classes!($first $($rest)*);
        all_tuple_classes!($($rest)*);
    };
    () => {};
}

all_tuple_classes!(C8 C7 C6 C5 C4 C3 C2 C1);
