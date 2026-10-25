use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::icon::Icon;
use crate::prelude::Unset;

const ICON_BUTTON: &str = "md-icon-button";

/// An icon-only button. `label` is required: it's the accessible name and tooltip.
pub fn icon_button() -> IconButton<Unset, Unset, Inert> {
    IconButton {
        icon: None,
        label: None,
        action: Inert,
    }
}

pub struct IconButton<I, L, A> {
    icon: Option<I>,
    label: Option<L>,
    action: A,
}

impl<I, L, A> IconButton<I, L, A> {
    pub fn icon<I2: Icon>(self, icon: I2) -> IconButton<I2, L, A> {
        IconButton {
            icon: Some(icon),
            label: self.label,
            action: self.action,
        }
    }

    pub fn label<L2: AttrValue + Clone>(self, label: L2) -> IconButton<I, L2, A> {
        IconButton {
            icon: self.icon,
            label: Some(label),
            action: self.action,
        }
    }
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
        self.action.element(ICON_BUTTON, self.label, self.icon)
    }
}
impl<I: Icon, L: AttrValue + Clone, A: Action> RunTimeHtmlSizeHint for IconButton<I, L, A> {
    fn size_hint(&self) -> usize {
        // `aria-label` and `title`, and the icon's name.
        2 * self.label.size_hint() + self.icon.size_hint()
    }
}
