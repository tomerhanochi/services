use htmlb::Component;
use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::class::{
    LIST_ITEM, LIST_ITEM_HEADLINE, LIST_ITEM_LEADING, LIST_ITEM_MAIN, LIST_ITEM_SUPPORTING,
    LIST_ITEM_TEXT, LIST_ITEM_TRAILING, LIST_ITEM_TRAILING_TEXT,
};

/// One row of a [`list`](crate::list). With an action, the whole row (except [`trailing`](Self::trailing))
/// is the target, which is what a thumb expects on a phone.
pub fn list_item<H: IntoHtml>(headline: H) -> ListItem<H, (), (), (), (), Inert> {
    ListItem {
        headline,
        leading: None,
        supporting: None,
        trailing_text: None,
        trailing: None,
        action: Inert,
    }
}

pub struct ListItem<H, L, S, T, X, A> {
    headline: H,
    leading: Option<L>,
    supporting: Option<S>,
    trailing_text: Option<T>,
    trailing: Option<X>,
    action: A,
}

impl<H, L, S, T, X, A> ListItem<H, L, S, T, X, A> {
    /// Before the text: usually an icon, e.g. [`icon::Folder`](crate::icon::Folder).
    pub fn leading<L2: IntoHtml>(self, leading: L2) -> ListItem<H, L2, S, T, X, A> {
        let ListItem {
            headline,
            supporting,
            trailing_text,
            trailing,
            action,
            ..
        } = self;
        ListItem {
            headline,
            leading: Some(leading),
            supporting,
            trailing_text,
            trailing,
            action,
        }
    }

    /// A second line under the headline.
    pub fn supporting<S2: IntoHtml>(self, supporting: S2) -> ListItem<H, L, S2, T, X, A> {
        let ListItem {
            headline,
            leading,
            trailing_text,
            trailing,
            action,
            ..
        } = self;
        ListItem {
            headline,
            leading,
            supporting: Some(supporting),
            trailing_text,
            trailing,
            action,
        }
    }

    /// Short text at the end of the row, part of the row's target (a size, a count).
    pub fn trailing_text<T2: IntoHtml>(self, text: T2) -> ListItem<H, L, S, T2, X, A> {
        let ListItem {
            headline,
            leading,
            supporting,
            trailing,
            action,
            ..
        } = self;
        ListItem {
            headline,
            leading,
            supporting,
            trailing_text: Some(text),
            trailing,
            action,
        }
    }

    /// A control at the end of the row with its own action, e.g. an [`icon_button`](crate::icon_button)
    /// opening a [`sheet`](crate::sheet).
    pub fn trailing<X2: IntoHtml>(self, trailing: X2) -> ListItem<H, L, S, T, X2, A> {
        let ListItem {
            headline,
            leading,
            supporting,
            trailing_text,
            action,
            ..
        } = self;
        ListItem {
            headline,
            leading,
            supporting,
            trailing_text,
            trailing: Some(trailing),
            action,
        }
    }
}

impl<H, L, S, T, X> Actionable for ListItem<H, L, S, T, X, Inert> {
    type With<A: Action> = ListItem<H, L, S, T, X, A>;

    fn action<A: Action>(self, action: A) -> ListItem<H, L, S, T, X, A> {
        let ListItem {
            headline,
            leading,
            supporting,
            trailing_text,
            trailing,
            ..
        } = self;
        ListItem {
            headline,
            leading,
            supporting,
            trailing_text,
            trailing,
            action,
        }
    }
}

impl<H: IntoHtml, L: IntoHtml, S: IntoHtml, T: IntoHtml, X: IntoHtml, A: Action> Component
    for ListItem<H, L, S, T, X, A>
{
    fn render(self) -> impl IntoHtml {
        let main = (
            self.leading
                .map(|l| span().class(LIST_ITEM_LEADING).child(l)),
            span().class(LIST_ITEM_TEXT).child((
                span().class(LIST_ITEM_HEADLINE).child(self.headline),
                self.supporting
                    .map(|s| span().class(LIST_ITEM_SUPPORTING).child(s)),
            )),
            self.trailing_text
                .map(|t| span().class(LIST_ITEM_TRAILING_TEXT).child(t)),
        );
        li().class(LIST_ITEM).child((
            self.action.element(LIST_ITEM_MAIN, None::<&str>, main),
            self.trailing
                .map(|x| div().class(LIST_ITEM_TRAILING).child(x)),
        ))
    }
    fn dynamic_len_hint(&self) -> usize {
        self.headline.len_hint()
            + self.leading.as_ref().map_or(0, L::len_hint)
            + self.supporting.as_ref().map_or(0, S::len_hint)
            + self.trailing_text.as_ref().map_or(0, T::len_hint)
            + self.trailing.as_ref().map_or(0, X::len_hint)
    }
}
