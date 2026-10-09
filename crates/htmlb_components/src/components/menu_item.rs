use htmlb::Component;
use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::icon::Icon;

const MENU_ITEM: &str = "md-menu-item";

/// A row in a [`sheet`](crate::sheet) or menu: an icon and a label, with an action.
pub fn menu_item<I: Icon, L: IntoHtml>(icon: I, label: L) -> MenuItem<I, L, Inert> {
    MenuItem {
        icon,
        label,
        action: Inert,
    }
}

pub struct MenuItem<I, L, A> {
    icon: I,
    label: L,
    action: A,
}

impl<I, L> Actionable for MenuItem<I, L, Inert> {
    type With<A: Action> = MenuItem<I, L, A>;

    fn action<A: Action>(self, action: A) -> MenuItem<I, L, A> {
        MenuItem {
            icon: self.icon,
            label: self.label,
            action,
        }
    }
}

impl<I: Icon, L: IntoHtml, A: Action> Component for MenuItem<I, L, A> {
    fn render(self) -> impl IntoHtml {
        self.action.element(
            MENU_ITEM,
            None::<&str>,
            (self.icon, span().child(self.label)),
        )
    }
    fn dynamic_len_hint(&self) -> usize {
        self.label.len_hint()
    }
}
