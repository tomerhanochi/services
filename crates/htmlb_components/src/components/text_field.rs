use htmlb::prelude::*;
use htmlb::{AttrValue, Component};

use crate::class::{
    TEXT_FIELD, TEXT_FIELD_ERROR, TEXT_FIELD_INPUT, TEXT_FIELD_LABEL, TEXT_FIELD_SUPPORTING,
};

/// An outlined text field whose label floats above the value.
pub fn text_field<L: IntoHtml, N: AttrValue>(
    label: L,
    name: N,
) -> TextField<L, N, &'static str, ()> {
    TextField {
        label,
        name,
        value: None,
        supporting: None,
        error: false,
        required: false,
        autofocus: false,
        verbatim: false,
    }
}

pub struct TextField<L, N, V, S> {
    label: L,
    name: N,
    value: Option<V>,
    supporting: Option<S>,
    error: bool,
    required: bool,
    autofocus: bool,
    verbatim: bool,
}

impl<L, N, V, S> TextField<L, N, V, S> {
    pub fn value<V2: AttrValue>(self, value: V2) -> TextField<L, N, V2, S> {
        let TextField {
            label,
            name,
            supporting,
            error,
            required,
            autofocus,
            verbatim,
            ..
        } = self;
        TextField {
            label,
            name,
            value: Some(value),
            supporting,
            error,
            required,
            autofocus,
            verbatim,
        }
    }

    /// Help text under the field.
    pub fn supporting<S2: IntoHtml>(self, supporting: S2) -> TextField<L, N, V, S2> {
        let TextField {
            label,
            name,
            value,
            error,
            required,
            autofocus,
            verbatim,
            ..
        } = self;
        TextField {
            label,
            name,
            value,
            supporting: Some(supporting),
            error,
            required,
            autofocus,
            verbatim,
        }
    }

    /// Shows the field, and its supporting text, as invalid.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    /// For names, codes and paths: no auto-capitalization, autocorrect or spell-check.
    pub fn verbatim(mut self) -> Self {
        self.verbatim = true;
        self
    }
}

impl<L: IntoHtml, N: AttrValue, V: AttrValue, S: IntoHtml> Component for TextField<L, N, V, S> {
    fn render(self) -> impl IntoHtml {
        let input = input()
            .class(TEXT_FIELD_INPUT)
            .r#type(InputType::Text)
            .name(self.name)
            .value(self.value)
            // The label floats while the placeholder isn't shown (`:placeholder-shown`).
            .placeholder(" ")
            .required(self.required)
            .autofocus(self.autofocus)
            .aria_invalid(self.error.then_some("true"))
            .autocapitalize(self.verbatim.then_some(AutoCapitalize::Off))
            .spellcheck(self.verbatim.then_some(false))
            .attr("autocorrect", self.verbatim.then_some("off"));
        label()
            .class((TEXT_FIELD, self.error.then_some(TEXT_FIELD_ERROR)))
            .child((
                input,
                span().class(TEXT_FIELD_LABEL).child(self.label),
                self.supporting
                    .map(|s| span().class(TEXT_FIELD_SUPPORTING).child(s)),
            ))
    }
    fn dynamic_len_hint(&self) -> usize {
        self.label.len_hint()
            + self.name.value_len_hint()
            + self.value.as_ref().map_or(0, V::value_len_hint)
            + self.supporting.as_ref().map_or(0, S::len_hint)
    }
}
