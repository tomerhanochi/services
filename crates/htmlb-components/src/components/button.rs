use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::icon::Icon;
use crate::prelude::Unset;

const BUTTON: &str = "md-button";
const BUTTON_LABEL: &str = "md-button__label";

const fn style_class(style: ButtonStyle) -> &'static str {
    match style {
        ButtonStyle::Filled => "md-button-filled",
        ButtonStyle::Tonal => "md-button-tonal",
        ButtonStyle::Elevated => "md-button-elevated",
        ButtonStyle::Outlined => "md-button-outlined",
        ButtonStyle::Text => "md-button-text",
    }
}

/// Emphasis, from highest to lowest: `Filled`, `Tonal`, `Elevated`, `Outlined`, `Text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonStyle {
    Filled,
    Tonal,
    Elevated,
    Outlined,
    Text,
}

impl ButtonStyle {
    pub const ALL: &[ButtonStyle] = &[
        ButtonStyle::Filled,
        ButtonStyle::Tonal,
        ButtonStyle::Elevated,
        ButtonStyle::Outlined,
        ButtonStyle::Text,
    ];
}

/// The highest-emphasis button: the screen's main action.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn filled_button() -> Button<Unset, (), Inert> {
    new_button(ButtonStyle::Filled)
}

/// A secondary action that still stands out.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn tonal_button() -> Button<Unset, (), Inert> {
    new_button(ButtonStyle::Tonal)
}

/// A filled-tonal button lifted off a patterned background.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn elevated_button() -> Button<Unset, (), Inert> {
    new_button(ButtonStyle::Elevated)
}

/// A medium-emphasis action, e.g. "Cancel" next to a filled "Save".
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn outlined_button() -> Button<Unset, (), Inert> {
    new_button(ButtonStyle::Outlined)
}

/// The lowest emphasis: dialog actions, inline links.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn text_button() -> Button<Unset, (), Inert> {
    new_button(ButtonStyle::Text)
}

fn new_button(style: ButtonStyle) -> Button<Unset, (), Inert> {
    Button {
        style,
        label: Unset,
        icon: None,
        action: Inert,
    }
}

pub struct Button<L, I, A> {
    style: ButtonStyle,
    label: L,
    icon: Option<I>,
    action: A,
}

impl<L, I, A> Button<L, I, A> {
    /// The button's visible label.
    pub fn label<L2: IntoHtml>(self, label: L2) -> Button<L2, I, A> {
        let Button {
            style,
            icon,
            action,
            ..
        } = self;
        Button {
            style,
            label,
            icon,
            action,
        }
    }

    /// An icon before the label.
    pub fn icon<I2: Icon>(self, icon: I2) -> Button<L, I2, A> {
        Button {
            style: self.style,
            label: self.label,
            icon: Some(icon),
            action: self.action,
        }
    }
}

impl<L, I> Actionable for Button<L, I, Inert> {
    type With<A: Action> = Button<L, I, A>;

    fn action<A: Action>(self, action: A) -> Button<L, I, A> {
        Button {
            style: self.style,
            label: self.label,
            icon: self.icon,
            action,
        }
    }
}

impl<L: IntoHtml, I: IntoHtml, A: Action> Component for Button<L, I, A> {
    fn render(self) -> impl IntoHtml {
        self.action.element(
            (BUTTON, style_class(self.style)),
            None::<&str>,
            (self.icon, span().class(BUTTON_LABEL).child(self.label)),
        )
    }
}
impl<L: IntoHtml, I: IntoHtml, A: Action> RunTimeHtmlSizeHint for Button<L, I, A> {
    fn size_hint(&self) -> usize {
        self.label.size_hint() + self.icon.size_hint()
    }
}
