use htmlb::prelude::*;
use htmlb::{AttrValue, Component};

const CHECKBOX: &str = "md-checkbox";

/// A checkbox submitting `name=value` when checked. Put it in a row whose action is
/// [`label`](crate::Actionable::label), so the whole row toggles it.
pub fn checkbox<N: AttrValue, V: AttrValue>(name: N, value: V) -> Checkbox<N, V, ()> {
    Checkbox {
        name,
        value,
        form: None,
    }
}

pub struct Checkbox<N, V, F> {
    name: N,
    value: V,
    form: Option<F>,
}

impl<N, V, F> Checkbox<N, V, F> {
    /// Submitted with the form with this id, rather than the enclosing one.
    pub fn form<F2: AttrValue>(self, form: F2) -> Checkbox<N, V, F2> {
        Checkbox {
            name: self.name,
            value: self.value,
            form: Some(form),
        }
    }
}

impl<N: AttrValue, V: AttrValue, F: AttrValue> Component for Checkbox<N, V, F> {
    fn render(self) -> impl IntoHtml {
        input()
            .class(CHECKBOX)
            .r#type(InputType::Checkbox)
            .name(self.name)
            .value(self.value)
            .form(self.form)
    }
    fn dynamic_len_hint(&self) -> usize {
        self.name.value_len_hint()
            + self.value.value_len_hint()
            + self.form.as_ref().map_or(0, F::value_len_hint)
    }
}
