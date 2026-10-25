use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::icon::Icon;
use crate::prelude::Unset;

const MENU_ITEM: &str = "md-menu-item";

/// A row in a [`sheet`](crate::sheet) or menu: an icon and a label, with an action.
pub fn menu_item() -> MenuItem<Unset, Unset, Inert> {
    MenuItem {
        icon: None,
        label: Unset,
        action: Inert,
    }
}

pub struct MenuItem<I, L, A> {
    icon: Option<I>,
    label: L,
    action: A,
}

impl<I, L, A> MenuItem<I, L, A> {
    pub fn icon<I2: Icon>(self, icon: I2) -> MenuItem<I2, L, A> {
        MenuItem {
            icon: Some(icon),
            label: self.label,
            action: self.action,
        }
    }

    pub fn label<L2: IntoHtml>(self, label: L2) -> MenuItem<I, L2, A> {
        MenuItem {
            icon: self.icon,
            label,
            action: self.action,
        }
    }
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
}
impl<I: Icon, L: IntoHtml, A: Action> RunTimeHtmlSizeHint for MenuItem<I, L, A> {
    fn size_hint(&self) -> usize {
        self.label.size_hint() + self.icon.size_hint()
    }
}
