//! What an interactive component does, and therefore which element it is.

use htmlb::prelude::*;
use htmlb::{AttrValue, ClassList};

/// Renders an interactive component as the element that natively does what it should:
/// `<a href>` for navigation, `<button type=submit>` for forms, `<button popovertarget>`
/// for popovers. `label`, when given, becomes `aria-label` and `title` (for icon-only
/// controls).
pub trait Action {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml;
}

/// No action yet: a `<div>`. A component left in this state is decoration.
pub struct Inert;

/// Navigates to `href`.
pub struct Link<H>(pub H);

/// Downloads `href`.
pub struct Download<H>(pub H);

/// Submits the enclosing form, or the form with id `form`.
pub struct Submit<F = &'static str> {
    pub form: Option<F>,
}

/// Shows the popover with this id (a [`sheet`](crate::sheet)).
pub struct ShowPopover<I>(pub I);

/// Hides the popover with this id.
pub struct HidePopover<I>(pub I);

impl Action for Inert {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        _: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        div().class(class).child(children)
    }
}

impl<H: AttrValue> Action for Link<H> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        a().class(class)
            .href(self.0)
            .aria_label(label.clone())
            .title(label)
            .child(children)
    }
}

impl<H: AttrValue> Action for Download<H> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        a().class(class)
            .href(self.0)
            .download("")
            .aria_label(label.clone())
            .title(label)
            .child(children)
    }
}

impl<F: AttrValue> Action for Submit<F> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        button()
            .class(class)
            .r#type(ButtonType::Submit)
            .form(self.form)
            .aria_label(label.clone())
            .title(label)
            .child(children)
    }
}

impl<I: AttrValue> Action for ShowPopover<I> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        button()
            .class(class)
            .r#type(ButtonType::Button)
            .popovertarget(self.0)
            .popovertargetaction(PopoverTargetAction::Show)
            .aria_label(label.clone())
            .title(label)
            .child(children)
    }
}

impl<I: AttrValue> Action for HidePopover<I> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        button()
            .class(class)
            .r#type(ButtonType::Button)
            .popovertarget(self.0)
            .popovertargetaction(PopoverTargetAction::Hide)
            .aria_label(label.clone())
            .title(label)
            .child(children)
    }
}

/// Either action, chosen at runtime: e.g. a link for folders, a download for files.
impl<A: Action, B: Action> Action for Either<A, B> {
    fn element<K: ClassList, L: AttrValue + Clone, C: IntoHtml>(
        self,
        class: K,
        label: Option<L>,
        children: C,
    ) -> impl IntoHtml {
        match self {
            Either::Left(a) => Either::Left(a.element(class, label, children)),
            Either::Right(b) => Either::Right(b.element(class, label, children)),
        }
    }
}

/// The builder methods that pick an [`Action`], for components that don't have one yet.
/// Implementors only provide [`action`](Self::action); the rest are shorthands for it.
pub trait Actionable: Sized {
    /// The same component, doing `A`.
    type With<A: Action>;

    /// Give it any [`Action`], e.g. an `Either` of two decided at runtime.
    fn action<A: Action>(self, action: A) -> Self::With<A>;

    /// Make it a link to `href`.
    fn href<H: AttrValue>(self, href: H) -> Self::With<Link<H>> {
        self.action(Link(href))
    }

    /// Make it a link that downloads `href`.
    fn download<H: AttrValue>(self, href: H) -> Self::With<Download<H>> {
        self.action(Download(href))
    }

    /// Make it submit the form it's in.
    fn submit(self) -> Self::With<Submit> {
        self.action(Submit { form: None })
    }

    /// Make it submit the form with id `form`, wherever it is.
    fn submit_form<F: AttrValue>(self, form: F) -> Self::With<Submit<F>> {
        self.action(Submit { form: Some(form) })
    }

    /// Make it show the popover (e.g. [`sheet`](crate::sheet)) with id `id`.
    fn opens<I: AttrValue>(self, id: I) -> Self::With<ShowPopover<I>> {
        self.action(ShowPopover(id))
    }

    /// Make it hide the popover with id `id`.
    fn closes<I: AttrValue>(self, id: I) -> Self::With<HidePopover<I>> {
        self.action(HidePopover(id))
    }
}
