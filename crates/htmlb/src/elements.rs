//! Elements: `El<Tag, Attributes, Children>` and every HTML element.

use std::marker::PhantomData;

use crate::IntoHtml;
use crate::class::{ClassList, Classes};
use crate::attrs::{Attr, AttrValue, Attribute, BoolAttr, DynAttr, Typed, enums::*, keys as attr};

/// What an element does with children.
pub mod kind {
    /// Any [`IntoHtml`](crate::IntoHtml) children, text escaped.
    pub struct Normal;
    /// No children and no closing tag: `<img>`, `<br>`, `<input>`, ...
    pub struct Void;
    /// String children written verbatim: `<script>`, `<style>`.
    pub struct RawText;
}

use kind::*;

pub trait Kind {
    const VOID: bool = false;
}
impl Kind for Normal {}
impl Kind for RawText {}
impl Kind for Void {
    const VOID: bool = true;
}

/// Element kinds that have a `.child()` method (all but void elements).
#[diagnostic::on_unimplemented(
    message = "void elements (`<img>`, `<br>`, `<input>`, ...) can't have children",
    label = "this element is void"
)]
pub trait HasChildren: Kind {}
impl HasChildren for Normal {}
impl HasChildren for RawText {}

/// A value that can be a child of an element of kind `K`, and what gets stored.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a child of this element",
    label = "`<script>`/`<style>` take strings only (written unescaped); other elements take any `IntoHtml`"
)]
pub trait Child<K> {
    type Out: IntoHtml;
    fn into_child(self) -> Self::Out;
}

impl<N: IntoHtml> Child<Normal> for N {
    type Out = N;
    #[inline]
    fn into_child(self) -> N {
        self
    }
}

impl<S: AsRef<str>> Child<RawText> for S {
    type Out = Unescaped<S>;
    #[inline]
    fn into_child(self) -> Unescaped<S> {
        Unescaped(self)
    }
}

/// A `<script>`/`<style>` child, written verbatim. Never put untrusted input here.
pub struct Unescaped<S>(S);

impl<S: AsRef<str>> IntoHtml for Unescaped<S> {
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

/// An element name, as a type. The strings are concatenated at compile time.
pub trait Tag {
    /// `div`
    const NAME: &'static str;
    /// `<div`
    const OPEN: &'static str;
    /// `<div>`, used when there are no attributes
    const OPEN_GT: &'static str;
    /// `</div>`
    const CLOSE: &'static str;
    type Kind: Kind;
}

/// An element: tag `T`, attribute list `A`, children `C`.
#[must_use = "an element does nothing until rendered"]
pub struct El<T, A, C> {
    attrs: A,
    children: C,
    tag: PhantomData<fn() -> T>,
}

impl<T, A, C> El<T, A, C> {
    #[inline]
    fn push_attr<X>(self, attr: X) -> El<T, (A, X), C> {
        El { attrs: (self.attrs, attr), children: self.children, tag: PhantomData }
    }
}

impl<T: Tag> El<T, (), ()> {
    #[inline]
    fn new() -> Self {
        El { attrs: (), children: (), tag: PhantomData }
    }
}

impl<T: Tag, A, C> El<T, A, C> {
    /// Appends a child: anything [`IntoHtml`] for normal elements, strings for
    /// `<script>`/`<style>`. Void elements have no children.
    #[inline]
    pub fn child<N: Child<T::Kind>>(self, child: N) -> El<T, A, (C, N::Out)>
    where
        T::Kind: HasChildren,
    {
        El { attrs: self.attrs, children: (self.children, child.into_child()), tag: PhantomData }
    }

    /// Any attribute by name: `data-*`, custom elements, anything not modelled here.
    /// The name must be `&'static str`, so it can't come from request data.
    #[inline]
    pub fn attr<V: AttrValue>(self, name: &'static str, value: V) -> El<T, (A, DynAttr<V>), C> {
        self.push_attr(DynAttr::new(name, value))
    }
}

impl<T: Tag, A: Attribute, C: IntoHtml> IntoHtml for El<T, A, C> {
    const MIN_LEN: usize = if <T::Kind as Kind>::VOID {
        T::OPEN.len() + 1 + A::ATTRS_MIN_LEN
    } else {
        T::OPEN.len() + 1 + A::ATTRS_MIN_LEN + C::MIN_LEN + T::CLOSE.len()
    };

    #[inline]
    fn len_hint(&self) -> usize {
        let open = T::OPEN.len() + 1 + self.attrs.attrs_len_hint();
        if <T::Kind as Kind>::VOID { open } else { open + self.children.len_hint() + T::CLOSE.len() }
    }


    #[inline]
    fn write_html(self, buf: &mut String) {
        if A::EMPTY {
            buf.push_str(T::OPEN_GT);
        } else {
            buf.push_str(T::OPEN);
            self.attrs.write_attrs(buf);
            buf.push('>');
        }
        if !<T::Kind as Kind>::VOID {
            self.children.write_html(buf);
            buf.push_str(T::CLOSE);
        }
    }
}

/// One attribute method. `$tag` is the element type, or `T` for global attributes.
macro_rules! attr_method {
    ($tag:ty; $m:ident : Text) => {
        #[doc = concat!("The `", stringify!($m), "` attribute.")]
        #[inline]
        pub fn $m<V: AttrValue>(self, value: V) -> El<$tag, (A, Attr<attr::$m, V>), C> {
            self.push_attr(Attr::new(value))
        }
    };
    ($tag:ty; $m:ident : Bool) => {
        #[doc = concat!("The `", stringify!($m), "` boolean attribute: present iff `on`.")]
        #[inline]
        pub fn $m(self, on: bool) -> El<$tag, (A, BoolAttr<attr::$m>), C> {
            self.push_attr(BoolAttr::new(on))
        }
    };
    ($tag:ty; $m:ident : Class) => {
        #[doc = concat!("The `", stringify!($m), "` attribute: a [`ClassList`], joined with spaces.")]
        #[inline]
        pub fn $m<V: ClassList>(self, value: V) -> El<$tag, (A, Attr<attr::$m, Classes<V>>), C> {
            self.push_attr(Attr::new(Classes(value)))
        }
    };
    ($tag:ty; $m:ident : $ty:ty) => {
        #[doc = concat!("The `", stringify!($m), "` attribute. Takes `", stringify!($ty), "` or `Option<", stringify!($ty), ">`.")]
        #[inline]
        pub fn $m<V: Typed<$ty>>(self, value: V) -> El<$tag, (A, Attr<attr::$m, V>), C> {
            self.push_attr(Attr::new(value))
        }
    };
}

macro_rules! elements {
    ($( $name:ident : $elkind:ident { $($attr:ident : $kind:tt),* $(,)? } )*) => {
        /// Element name types.
        pub mod tag {
            $(
                #[allow(non_camel_case_types)]
                #[derive(Debug, Clone, Copy)]
                pub struct $name;
            )*
        }

        /// One constructor function per element: `div()`, `a()`, ...
        pub mod constructors {
            use super::*;
            $(
                #[doc = concat!("`<", stringify!($name), ">`")]
                #[inline]
                pub fn $name() -> El<tag::$name, (), ()> {
                    El::new()
                }
            )*
        }

        $(
            impl Tag for tag::$name {
                const NAME: &'static str = stringify!($name);
                const OPEN: &'static str = concat!("<", stringify!($name));
                const OPEN_GT: &'static str = concat!("<", stringify!($name), ">");
                const CLOSE: &'static str = concat!("</", stringify!($name), ">");
                type Kind = $elkind;
            }

            impl<A, C> El<tag::$name, A, C> {
                $(attr_method!(tag::$name; $attr : $kind);)*
            }
        )*
    };
}

macro_rules! global_attributes {
    ($($attr:ident : $kind:tt),* $(,)?) => {
        /// Global attributes, available on every element.
        impl<T: Tag, A, C> El<T, A, C> {
            $(attr_method!(T; $attr : $kind);)*
        }
    };
}

include!("element_defs.rs");

pub use constructors::*;
