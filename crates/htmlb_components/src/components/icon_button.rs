use htmlb::prelude::*;
use htmlb::{AttrValue, Component};

use crate::action::{Action, Actionable, Inert};
use crate::class::ICON_BUTTON;
use crate::icon::Icon;

/// An icon-only button. `label` is required: it's the accessible name and tooltip.
pub fn icon_button<I: Icon, L: AttrValue + Clone>(icon: I, label: L) -> IconButton<I, L, Inert> {
    IconButton {
        icon,
        label,
        action: Inert,
    }
}

pub struct IconButton<I, L, A> {
    icon: I,
    label: L,
    action: A,
}

impl<I, L> Actionable for IconButton<I, L, Inert> {
    type With<A: Action> = IconButton<I, L, A>;

    fn action<A: Action>(self, action: A) -> IconButton<I, L, A> {
        IconButton {
            icon: self.icon,
            label: self.label,
            action,
        }
    }
}

impl<I: Icon, L: AttrValue + Clone, A: Action> Component for IconButton<I, L, A> {
    fn render(self) -> impl IntoHtml {
        self.action
            .element(ICON_BUTTON, Some(self.label), self.icon)
    }
    fn dynamic_len_hint(&self) -> usize {
        // `aria-label` and `title`. The icon is in `MIN_LEN`.
        2 * self.label.value_len_hint()
    }
}
