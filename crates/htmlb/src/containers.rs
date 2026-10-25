//! Containers: tuples, `Option`, `Vec`, arrays, `Either`, and iterators. (`Box` is in
//! [`component`](crate::component).)

use crate::{CompileTimeHtmlSizeHint, IntoHtml, RunTimeHtmlSizeHint};

impl CompileTimeHtmlSizeHint for () {
    const SIZE_HINT: usize = 0;
}
impl RunTimeHtmlSizeHint for () {
    fn size_hint(&self) -> usize {
        0
    }
}
impl IntoHtml for () {
    #[inline]
    fn write_html(self, _: &mut String) {}
}

macro_rules! tuples {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: CompileTimeHtmlSizeHint),+> CompileTimeHtmlSizeHint for ($($t,)+) {
            const SIZE_HINT: usize = 0 $(+ <$t as CompileTimeHtmlSizeHint>::SIZE_HINT)+;
        }
        #[allow(non_snake_case)]
        impl<$($t: RunTimeHtmlSizeHint),+> RunTimeHtmlSizeHint for ($($t,)+) {
            #[inline]
            fn size_hint(&self) -> usize {
                let ($($t,)+) = self;
                0 $(+ <$t as RunTimeHtmlSizeHint>::size_hint($t))+
            }
        }
        #[allow(non_snake_case)]
        impl<$($t: IntoHtml),+> IntoHtml for ($($t,)+) {
            #[inline]
            fn write_html(self, buf: &mut String) {
                let ($($t,)+) = self;
                $(<$t as IntoHtml>::write_html($t, buf);)+
            }
        }
    };
}

macro_rules! all_tuples {
    ($first:ident $($rest:ident)*) => {
        tuples!($first $($rest)*);
        all_tuples!($($rest)*);
    };
    () => {};
}

all_tuples!(T16 T15 T14 T13 T12 T11 T10 T9 T8 T7 T6 T5 T4 T3 T2 T1);

impl<T: CompileTimeHtmlSizeHint> CompileTimeHtmlSizeHint for Option<T> {
    const SIZE_HINT: usize = 0;
}
impl<T: CompileTimeHtmlSizeHint + RunTimeHtmlSizeHint> RunTimeHtmlSizeHint for Option<T> {
    #[inline]
    fn size_hint(&self) -> usize {
        self.as_ref()
            .map_or(0, |value| T::SIZE_HINT + value.size_hint())
    }
}
impl<T: IntoHtml> IntoHtml for Option<T> {
    #[inline]
    fn write_html(self, buf: &mut String) {
        if let Some(t) = self {
            t.write_html(buf)
        }
    }
}

impl<T: IntoHtml> CompileTimeHtmlSizeHint for Vec<T> {
    const SIZE_HINT: usize = 0;
}
impl<T: IntoHtml> RunTimeHtmlSizeHint for Vec<T> {
    #[inline]
    fn size_hint(&self) -> usize {
        self.iter()
            .map(|value| T::SIZE_HINT + value.size_hint())
            .sum()
    }
}
impl<T: IntoHtml> IntoHtml for Vec<T> {
    #[inline]
    fn write_html(self, buf: &mut String) {
        for t in self {
            t.write_html(buf)
        }
    }
}

impl<T: IntoHtml, const N: usize> CompileTimeHtmlSizeHint for [T; N] {
    const SIZE_HINT: usize = <T as CompileTimeHtmlSizeHint>::SIZE_HINT * N;
}
impl<T: IntoHtml, const N: usize> RunTimeHtmlSizeHint for [T; N] {
    #[inline]
    fn size_hint(&self) -> usize {
        self.iter().map(T::size_hint).sum()
    }
}
impl<T: IntoHtml, const N: usize> IntoHtml for [T; N] {
    #[inline]
    fn write_html(self, buf: &mut String) {
        for t in self {
            t.write_html(buf)
        }
    }
}

/// One of two different view types, e.g. for `if`/`else` branches.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: IntoHtml, R: IntoHtml> CompileTimeHtmlSizeHint for Either<L, R> {
    const SIZE_HINT: usize = if L::SIZE_HINT < R::SIZE_HINT {
        L::SIZE_HINT
    } else {
        R::SIZE_HINT
    };
}
impl<L: IntoHtml, R: IntoHtml> RunTimeHtmlSizeHint for Either<L, R> {
    #[inline]
    fn size_hint(&self) -> usize {
        match self {
            Either::Left(l) => L::SIZE_HINT + l.size_hint() - Self::SIZE_HINT,
            Either::Right(r) => R::SIZE_HINT + r.size_hint() - Self::SIZE_HINT,
        }
    }
}
impl<L: IntoHtml, R: IntoHtml> IntoHtml for Either<L, R> {
    #[inline]
    fn write_html(self, buf: &mut String) {
        match self {
            Either::Left(l) => l.write_html(buf),
            Either::Right(r) => r.write_html(buf),
        }
    }
}

/// Renders an iterator without collecting it. Its size estimate is only
/// `size_hint().0 × Item::SIZE_HINT`, because the items can be walked only once.
/// Create with [`IteratorExt::into_html`] or `IterHtml::from(iter)`.
pub struct IterHtml<I>(pub I);

impl<I: Iterator> From<I> for IterHtml<I> {
    fn from(iter: I) -> Self {
        IterHtml(iter)
    }
}

impl<I> CompileTimeHtmlSizeHint for IterHtml<I>
where
    I: Iterator,
    I::Item: IntoHtml,
{
    const SIZE_HINT: usize = 0;
}
impl<I> RunTimeHtmlSizeHint for IterHtml<I>
where
    I: Iterator,
    I::Item: IntoHtml,
{
    #[inline]
    fn size_hint(&self) -> usize {
        self.0.size_hint().0 * <I::Item as CompileTimeHtmlSizeHint>::SIZE_HINT
    }
}
impl<I> IntoHtml for IterHtml<I>
where
    I: Iterator,
    I::Item: IntoHtml,
{
    #[inline]
    fn write_html(self, buf: &mut String) {
        for item in self.0 {
            item.write_html(buf)
        }
    }
}

/// Renders a `Clone` iterator without collecting it. Size estimates walk a clone of the
/// iterator, building every item once more, so they are as precise as a `Vec`'s.
/// Create with [`IteratorExt::into_html_cloned`].
pub struct ClonedIterHtml<I>(pub I);

impl<I> CompileTimeHtmlSizeHint for ClonedIterHtml<I>
where
    I: Iterator + Clone,
    I::Item: IntoHtml,
{
    const SIZE_HINT: usize = 0;
}
impl<I> RunTimeHtmlSizeHint for ClonedIterHtml<I>
where
    I: Iterator + Clone,
    I::Item: IntoHtml,
{
    #[inline]
    fn size_hint(&self) -> usize {
        self.0
            .clone()
            .map(|item| <I::Item as CompileTimeHtmlSizeHint>::SIZE_HINT + item.size_hint())
            .sum()
    }
}
impl<I> IntoHtml for ClonedIterHtml<I>
where
    I: Iterator + Clone,
    I::Item: IntoHtml,
{
    #[inline]
    fn write_html(self, buf: &mut String) {
        for item in self.0 {
            item.write_html(buf)
        }
    }
}

/// `.into_html()` / `.into_html_cloned()` on every iterator.
pub trait IteratorExt: Iterator + Sized {
    /// Single pass; size estimate is `size_hint().0 × Item::SIZE_HINT`.
    fn into_html(self) -> IterHtml<Self> {
        IterHtml(self)
    }

    /// Size estimates walk a clone first; precise, but builds each item twice.
    fn into_html_cloned(self) -> ClonedIterHtml<Self>
    where
        Self: Clone,
    {
        ClonedIterHtml(self)
    }
}

impl<I: Iterator> IteratorExt for I {}
