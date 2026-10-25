use htmlb::Component;
use htmlb::prelude::*;

use crate::action::{Action, Actionable, Inert};
use crate::icon::Icon;

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
pub fn filled_button<L: IntoHtml>(label: L) -> Button<L, (), Inert> {
    new_button(ButtonStyle::Filled, label)
}

/// A secondary action that still stands out.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn tonal_button<L: IntoHtml>(label: L) -> Button<L, (), Inert> {
    new_button(ButtonStyle::Tonal, label)
}

/// A filled-tonal button lifted off a patterned background.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn elevated_button<L: IntoHtml>(label: L) -> Button<L, (), Inert> {
    new_button(ButtonStyle::Elevated, label)
}

/// A medium-emphasis action, e.g. "Cancel" next to a filled "Save".
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn outlined_button<L: IntoHtml>(label: L) -> Button<L, (), Inert> {
    new_button(ButtonStyle::Outlined, label)
}

/// The lowest emphasis: dialog actions, inline links.
///
/// Give it an action: [`href`](Actionable::href), [`submit`](Actionable::submit), ...
pub fn text_button<L: IntoHtml>(label: L) -> Button<L, (), Inert> {
    new_button(ButtonStyle::Text, label)
}

fn new_button<L: IntoHtml>(style: ButtonStyle, label: L) -> Button<L, (), Inert> {
    Button {
        style,
        label,
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
    fn dynamic_len_hint(&self) -> usize {
        self.label.len_hint() + self.icon.as_ref().map_or(0, I::len_hint)
    }
}
