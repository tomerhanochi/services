//! Attributes: a list of them per element, each with a typed value.

use std::borrow::Cow;
use std::marker::PhantomData;

use crate::{escape, num};

/// An attribute name, as a type. `PREFIX` (` name="`) and `BARE` (` name`) are
/// concatenated at compile time, so writing one is a single `push_str`.
pub trait AttrKey {
    const PREFIX: &'static str;
    const BARE: &'static str;
}

/// An element's attribute list: `()`, one attribute, or a pair `(earlier, newest)`.
pub trait Attribute {
    /// Bytes always written, from the type alone.
    const ATTRS_MIN_LEN: usize;
    /// True for `()`: lets `<div>` be written as one string instead of `<div` + `>`.
    const EMPTY: bool = false;
    fn attrs_len_hint(&self) -> usize;
    fn write_attrs(self, buf: &mut String);
}

impl Attribute for () {
    const ATTRS_MIN_LEN: usize = 0;
    const EMPTY: bool = true;
    #[inline]
    fn attrs_len_hint(&self) -> usize {
        0
    }
    #[inline]
    fn write_attrs(self, _: &mut String) {}
}

impl<A: Attribute, B: Attribute> Attribute for (A, B) {
    const ATTRS_MIN_LEN: usize = A::ATTRS_MIN_LEN + B::ATTRS_MIN_LEN;
    const EMPTY: bool = A::EMPTY && B::EMPTY;
    #[inline]
    fn attrs_len_hint(&self) -> usize {
        self.0.attrs_len_hint() + self.1.attrs_len_hint()
    }
    #[inline]
    fn write_attrs(self, buf: &mut String) {
        self.0.write_attrs(buf);
        self.1.write_attrs(buf);
    }
}

/// An attribute value. Written inside double quotes, escaped.
pub trait AttrValue {
    const VALUE_MIN_LEN: usize;
    /// False for `Option`: the whole attribute may be omitted.
    const ALWAYS_PRESENT: bool = true;
    #[inline]
    fn is_present(&self) -> bool {
        true
    }
    fn value_len_hint(&self) -> usize;
    fn write_value(self, buf: &mut String);
}

/// Values accepted by an attribute declared with type `T`: `T` itself, or `Option<T>`
/// to omit the attribute.
pub trait Typed<T>: AttrValue {}
impl<T: AttrValue> Typed<T> for T {}
impl<T: AttrValue> Typed<T> for Option<T> {}

/// ` name="value"`
pub struct Attr<K, V> {
    value: V,
    key: PhantomData<fn() -> K>,
}

impl<K, V> Attr<K, V> {
    #[inline]
    pub(crate) fn new(value: V) -> Self {
        Attr {
            value,
            key: PhantomData,
        }
    }
}

impl<K: AttrKey, V: AttrValue> Attribute for Attr<K, V> {
    const ATTRS_MIN_LEN: usize = if V::ALWAYS_PRESENT {
        K::PREFIX.len() + 1 + V::VALUE_MIN_LEN
    } else {
        0
    };
    #[inline]
    fn attrs_len_hint(&self) -> usize {
        if self.value.is_present() {
            K::PREFIX.len() + 1 + self.value.value_len_hint()
        } else {
            0
        }
    }
    #[inline]
    fn write_attrs(self, buf: &mut String) {
        if self.value.is_present() {
            buf.push_str(K::PREFIX);
            self.value.write_value(buf);
            buf.push('"');
        }
    }
}

/// A boolean attribute: ` name` when true, nothing when false.
pub struct BoolAttr<K> {
    on: bool,
    key: PhantomData<fn() -> K>,
}

impl<K> BoolAttr<K> {
    #[inline]
    pub(crate) fn new(on: bool) -> Self {
        BoolAttr {
            on,
            key: PhantomData,
        }
    }
}

impl<K: AttrKey> Attribute for BoolAttr<K> {
    const ATTRS_MIN_LEN: usize = 0;
    #[inline]
    fn attrs_len_hint(&self) -> usize {
        if self.on {
            K::BARE.len()
        } else {
            0
        }
    }
    #[inline]
    fn write_attrs(self, buf: &mut String) {
        if self.on {
            buf.push_str(K::BARE);
        }
    }
}

/// An attribute with a name chosen at the call site (`data-*`, custom elements, ...).
/// The name is `&'static str` so it can't come from request data, and is not escaped.
pub struct DynAttr<V> {
    name: &'static str,
    value: V,
}

impl<V> DynAttr<V> {
    #[inline]
    pub(crate) fn new(name: &'static str, value: V) -> Self {
        DynAttr { name, value }
    }
}

impl<V: AttrValue> Attribute for DynAttr<V> {
    // ` ` + `="` + `"`, plus the name at runtime.
    const ATTRS_MIN_LEN: usize = if V::ALWAYS_PRESENT {
        4 + V::VALUE_MIN_LEN
    } else {
        0
    };
    #[inline]
    fn attrs_len_hint(&self) -> usize {
        if self.value.is_present() {
            4 + self.name.len() + self.value.value_len_hint()
        } else {
            0
        }
    }
    #[inline]
    fn write_attrs(self, buf: &mut String) {
        if self.value.is_present() {
            buf.push(' ');
            buf.push_str(self.name);
            buf.push_str("=\"");
            self.value.write_value(buf);
            buf.push('"');
        }
    }
}

macro_rules! string_values {
    ($([$($gen:tt)*] $ty:ty => |$s:ident| $as_str:expr;)*) => {$(
        impl<$($gen)*> AttrValue for $ty {
            const VALUE_MIN_LEN: usize = 0;
            #[inline]
            fn value_len_hint(&self) -> usize {
                let $s = self;
                let s: &str = $as_str;
                s.len()
            }
            #[inline]
            fn write_value(self, buf: &mut String) {
                let $s = &self;
                escape::write(buf, $as_str)
            }
        }
    )*};
}

string_values! {
    ['a] &'a str => |s| s;
    [] String => |s| s.as_str();
    ['a] &'a String => |s| s.as_str();
    ['a] Cow<'a, str> => |s| s;
    [] Box<str> => |s| s;
}

macro_rules! int_values {
    ($($ty:ty)* => $wide:ty, $digits:path, $write:path) => {$(
        impl AttrValue for $ty {
            const VALUE_MIN_LEN: usize = 1;
            #[inline]
            fn value_len_hint(&self) -> usize {
                $digits(*self as $wide)
            }
            #[inline]
            fn write_value(self, buf: &mut String) {
                $write(buf, self as $wide)
            }
        }
    )*};
}

int_values!(u8 u16 u32 u64 usize => u64, num::digits, num::write_u64);
int_values!(i8 i16 i32 i64 isize => i64, num::digits_signed, num::write_i64);

macro_rules! display_values {
    ($($ty:ty)*) => {$(
        impl AttrValue for $ty {
            const VALUE_MIN_LEN: usize = 1;
            #[inline]
            fn value_len_hint(&self) -> usize {
                1
            }
            #[inline]
            fn write_value(self, buf: &mut String) {
                num::write_display(buf, self)
            }
        }
    )*};
}

display_values!(u128 i128 f32 f64);

/// `"true"` / `"false"`, for enumerated attributes like `draggable` and `aria-*`.
/// (Boolean attributes such as `disabled` take a plain `bool` and use [`BoolAttr`].)
impl AttrValue for bool {
    const VALUE_MIN_LEN: usize = 4;
    #[inline]
    fn value_len_hint(&self) -> usize {
        if *self {
            4
        } else {
            5
        }
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        buf.push_str(if self { "true" } else { "false" })
    }
}

impl AttrValue for char {
    const VALUE_MIN_LEN: usize = 1;
    #[inline]
    fn value_len_hint(&self) -> usize {
        self.len_utf8()
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        escape::write(buf, self.encode_utf8(&mut [0; 4]))
    }
}

/// `None` omits the whole attribute.
impl<V: AttrValue> AttrValue for Option<V> {
    const VALUE_MIN_LEN: usize = 0;
    const ALWAYS_PRESENT: bool = false;
    #[inline]
    fn is_present(&self) -> bool {
        self.is_some()
    }
    #[inline]
    fn value_len_hint(&self) -> usize {
        self.as_ref().map_or(0, V::value_len_hint)
    }
    #[inline]
    fn write_value(self, buf: &mut String) {
        if let Some(v) = self {
            v.write_value(buf)
        }
    }
}

/// Tuples concatenate: `.href(("/files/", name))`. Present if any part is present.
/// (`class` takes a [`ClassList`](crate::ClassList) instead, whose tuples are space-separated.)
macro_rules! tuple_values {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: AttrValue),+> AttrValue for ($($t,)+) {
            const VALUE_MIN_LEN: usize = 0 $(+ $t::VALUE_MIN_LEN)+;
            const ALWAYS_PRESENT: bool = false $(|| $t::ALWAYS_PRESENT)+;
            #[inline]
            fn is_present(&self) -> bool {
                let ($($t,)+) = self;
                false $(|| $t.is_present())+
            }
            #[inline]
            fn value_len_hint(&self) -> usize {
                let ($($t,)+) = self;
                0 $(+ $t.value_len_hint())+
            }
            #[inline]
            fn write_value(self, buf: &mut String) {
                let ($($t,)+) = self;
                $($t.write_value(buf);)+
            }
        }
    };
}

macro_rules! all_tuple_values {
    ($first:ident $($rest:ident)*) => {
        tuple_values!($first $($rest)*);
        all_tuple_values!($($rest)*);
    };
    () => {};
}

all_tuple_values!(V8 V7 V6 V5 V4 V3 V2 V1);

const fn min_len(values: &[&str]) -> usize {
    let mut min = usize::MAX;
    let mut i = 0;
    while i < values.len() {
        if values[i].len() < min {
            min = values[i].len();
        }
        i += 1;
    }
    min
}

macro_rules! attr_enums {
    ($(
        $(#[$meta:meta])*
        $name:ident { $($variant:ident = $s:literal),+ $(,)? }
    )*) => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
        }

        impl AttrValue for $name {
            const VALUE_MIN_LEN: usize = min_len(&[$($s),+]);
            #[inline]
            fn value_len_hint(&self) -> usize {
                self.as_str().len()
            }
            #[inline]
            fn write_value(self, buf: &mut String) {
                buf.push_str(self.as_str())
            }
        }
    )*};
}

/// Values for enumerated attributes. None of them need escaping.
pub mod enums {
    use super::*;

    attr_enums! {
        /// `<input type>`
        InputType {
            Button = "button", Checkbox = "checkbox", Color = "color", Date = "date",
            DatetimeLocal = "datetime-local", Email = "email", File = "file", Hidden = "hidden",
            Image = "image", Month = "month", Number = "number", Password = "password",
            Radio = "radio", Range = "range", Reset = "reset", Search = "search",
            Submit = "submit", Tel = "tel", Text = "text", Time = "time", Url = "url", Week = "week",
        }
        /// `<button type>`
        ButtonType { Submit = "submit", Reset = "reset", Button = "button" }
        /// `<form method>`, `formmethod`
        FormMethod { Get = "get", Post = "post", Dialog = "dialog" }
        /// `<form enctype>`, `formenctype`
        FormEnctype {
            UrlEncoded = "application/x-www-form-urlencoded",
            Multipart = "multipart/form-data",
            Plain = "text/plain",
        }
        /// `loading` on `<img>`/`<iframe>`
        Loading { Eager = "eager", Lazy = "lazy" }
        /// `<img decoding>`
        Decoding { Sync = "sync", Async = "async", Auto = "auto" }
        /// `crossorigin`
        CrossOrigin { Anonymous = "anonymous", UseCredentials = "use-credentials" }
        /// `referrerpolicy`
        ReferrerPolicy {
            NoReferrer = "no-referrer",
            NoReferrerWhenDowngrade = "no-referrer-when-downgrade",
            Origin = "origin",
            OriginWhenCrossOrigin = "origin-when-cross-origin",
            SameOrigin = "same-origin",
            StrictOrigin = "strict-origin",
            StrictOriginWhenCrossOrigin = "strict-origin-when-cross-origin",
            UnsafeUrl = "unsafe-url",
        }
        /// `fetchpriority`
        FetchPriority { High = "high", Low = "low", Auto = "auto" }
        /// `preload` on `<audio>`/`<video>`
        Preload { None = "none", Metadata = "metadata", Auto = "auto" }
        /// `dir`
        Dir { Ltr = "ltr", Rtl = "rtl", Auto = "auto" }
        /// `<th scope>`
        Scope { Row = "row", Col = "col", RowGroup = "rowgroup", ColGroup = "colgroup" }
        /// `<textarea wrap>`
        Wrap { Hard = "hard", Soft = "soft", Off = "off" }
        /// `<ol type>`
        OlType { Decimal = "1", LowerAlpha = "a", UpperAlpha = "A", LowerRoman = "i", UpperRoman = "I" }
        /// `<track kind>`
        TrackKind {
            Subtitles = "subtitles", Captions = "captions", Descriptions = "descriptions",
            Chapters = "chapters", Metadata = "metadata",
        }
        /// `autocapitalize`
        AutoCapitalize {
            Off = "off", None = "none", On = "on", Sentences = "sentences", Words = "words",
            Characters = "characters",
        }
        /// `enterkeyhint`
        EnterKeyHint {
            Enter = "enter", Done = "done", Go = "go", Next = "next", Previous = "previous",
            Search = "search", Send = "send",
        }
        /// `inputmode`
        InputMode {
            None = "none", Text = "text", Decimal = "decimal", Numeric = "numeric", Tel = "tel",
            Search = "search", Email = "email", Url = "url",
        }
        /// `contenteditable`
        ContentEditable { True = "true", False = "false", PlaintextOnly = "plaintext-only" }
        /// `translate`
        Translate { Yes = "yes", No = "no" }
        /// `popover`
        Popover { Auto = "auto", Manual = "manual", Hint = "hint" }
        /// `popovertargetaction`
        PopoverTargetAction { Toggle = "toggle", Show = "show", Hide = "hide" }
        /// `<button command>`: the built-in commands. (Custom `--commands` go through `.attr`.)
        Command {
            ShowModal = "show-modal", Close = "close", RequestClose = "request-close",
            ShowPopover = "show-popover", HidePopover = "hide-popover", TogglePopover = "toggle-popover",
        }
    }
}

/// Attribute name types, one per distinct attribute name.
pub mod keys {
    macro_rules! attr_keys {
        ($($key:ident $(= $name:literal)?),* $(,)?) => {
            $(attr_keys!(@one $key $(= $name)?);)*
        };
        (@one $key:ident) => {
            #[allow(non_camel_case_types)]
            pub struct $key;
            impl crate::AttrKey for $key {
                const PREFIX: &'static str = concat!(" ", stringify!($key), "=\"");
                const BARE: &'static str = concat!(" ", stringify!($key));
            }
        };
        (@one $key:ident = $name:literal) => {
            #[allow(non_camel_case_types)]
            pub struct $key;
            impl crate::AttrKey for $key {
                const PREFIX: &'static str = concat!(" ", $name, "=\"");
                const BARE: &'static str = concat!(" ", $name);
            }
        };
    }

    include!("attr_keys.rs");
}
