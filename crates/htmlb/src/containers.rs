//! Containers: tuples, `Option`, `Vec`, arrays, `Either`, and iterators. (`Box` is in
//! [`component`](crate::component).)

use crate::IntoHtml;

impl IntoHtml for () {
    const MIN_LEN: usize = 0;
    #[inline]
    fn write_html(self, _: &mut String) {}
}

macro_rules! tuples {
    ($($t:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($t: IntoHtml),+> IntoHtml for ($($t,)+) {
            const MIN_LEN: usize = 0 $(+ <$t as IntoHtml>::MIN_LEN)+;
            #[inline]
            fn len_hint(&self) -> usize {
                let ($($t,)+) = self;
                0 $(+ <$t as IntoHtml>::len_hint($t))+
            }
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

impl<T: IntoHtml> IntoHtml for Option<T> {
    const MIN_LEN: usize = 0;
    #[inline]
    fn len_hint(&self) -> usize {
        self.as_ref().map_or(0, T::len_hint)
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        if let Some(t) = self {
            t.write_html(buf)
        }
    }
}

impl<T: IntoHtml> IntoHtml for Vec<T> {
    const MIN_LEN: usize = 0;
    #[inline]
    fn len_hint(&self) -> usize {
        self.iter().map(T::len_hint).sum()
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        for t in self {
            t.write_html(buf)
        }
    }
}

impl<T: IntoHtml, const N: usize> IntoHtml for [T; N] {
    const MIN_LEN: usize = T::MIN_LEN * N;
    #[inline]
    fn len_hint(&self) -> usize {
        self.iter().map(T::len_hint).sum()
    }
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

impl<L: IntoHtml, R: IntoHtml> IntoHtml for Either<L, R> {
    const MIN_LEN: usize = if L::MIN_LEN < R::MIN_LEN { L::MIN_LEN } else { R::MIN_LEN };
    #[inline]
    fn len_hint(&self) -> usize {
        match self {
            Either::Left(l) => l.len_hint(),
            Either::Right(r) => r.len_hint(),
        }
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        match self {
            Either::Left(l) => l.write_html(buf),
            Either::Right(r) => r.write_html(buf),
        }
    }
}

/// Renders an iterator without collecting it. Its size estimate is only
/// `size_hint().0 × Item::MIN_LEN`, because the items can be walked only once.
/// Create with [`IteratorExt::into_html`] or `IterHtml::from(iter)`.
pub struct IterHtml<I>(pub I);

impl<I: Iterator> From<I> for IterHtml<I> {
    fn from(iter: I) -> Self {
        IterHtml(iter)
    }
}

impl<I> IntoHtml for IterHtml<I>
where
    I: Iterator,
    I::Item: IntoHtml,
{
    const MIN_LEN: usize = 0;
    #[inline]
    fn len_hint(&self) -> usize {
        self.0.size_hint().0 * <I::Item as IntoHtml>::MIN_LEN
    }
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

impl<I> IntoHtml for ClonedIterHtml<I>
where
    I: Iterator + Clone,
    I::Item: IntoHtml,
{
    const MIN_LEN: usize = 0;
    #[inline]
    fn len_hint(&self) -> usize {
        self.0.clone().map(|item| item.len_hint()).sum()
    }
    #[inline]
    fn write_html(self, buf: &mut String) {
        for item in self.0 {
            item.write_html(buf)
        }
    }
}

/// `.into_html()` / `.into_html_cloned()` on every iterator.
pub trait IteratorExt: Iterator + Sized {
    /// Single pass; size estimate is `size_hint().0 × Item::MIN_LEN`.
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
