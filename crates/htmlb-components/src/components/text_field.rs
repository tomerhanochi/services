use crate::prelude::Unset;
use htmlb::prelude::*;

const TEXT_FIELD: &str = "md-text-field";
const TEXT_FIELD_ERROR: &str = "md-text-field--error";
const TEXT_FIELD_INPUT: &str = "md-text-field__input";
const TEXT_FIELD_LABEL: &str = "md-text-field__label";
const TEXT_FIELD_SUPPORTING: &str = "md-text-field__supporting";

/// An outlined text field whose label floats above the value.
pub fn text_field() -> TextField<Unset, Unset, &'static str, ()> {
    TextField {
        label: Unset,
        name: None,
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
    name: Option<N>,
    value: Option<V>,
    supporting: Option<S>,
    error: bool,
    required: bool,
    autofocus: bool,
    verbatim: bool,
}

impl<L, N, V, S> TextField<L, N, V, S> {
    pub fn label<L2: IntoHtml>(self, label: L2) -> TextField<L2, N, V, S> {
        TextField {
            label,
            name: self.name,
            value: self.value,
            supporting: self.supporting,
            error: self.error,
            required: self.required,
            autofocus: self.autofocus,
            verbatim: self.verbatim,
        }
    }

    pub fn name<N2: AttrValue>(self, name: N2) -> TextField<L, N2, V, S> {
        TextField {
            label: self.label,
            name: Some(name),
            value: self.value,
            supporting: self.supporting,
            error: self.error,
            required: self.required,
            autofocus: self.autofocus,
            verbatim: self.verbatim,
        }
    }

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
}
impl<L: IntoHtml, N: AttrValue, V: AttrValue, S: IntoHtml> RunTimeHtmlSizeHint
    for TextField<L, N, V, S>
{
    fn size_hint(&self) -> usize {
        self.label.size_hint()
            + self.name.size_hint()
            + self.value.size_hint()
            + self.supporting.size_hint()
    }
}
