use crate::prelude::Unset;
use htmlb::prelude::*;

const CHECKBOX: &str = "md-checkbox";

/// A checkbox submitting `name=value` when checked. Put it in a row whose action is
/// [`label`](crate::Actionable::label), so the whole row toggles it.
pub fn checkbox() -> Checkbox<Unset, Unset, ()> {
    Checkbox {
        name: None,
        value: None,
        form: None,
    }
}

pub struct Checkbox<N, V, F> {
    name: Option<N>,
    value: Option<V>,
    form: Option<F>,
}

impl<N, V, F> Checkbox<N, V, F> {
    pub fn name<N2: AttrValue>(self, name: N2) -> Checkbox<N2, V, F> {
        Checkbox {
            name: Some(name),
            value: self.value,
            form: self.form,
        }
    }

    pub fn value<V2: AttrValue>(self, value: V2) -> Checkbox<N, V2, F> {
        Checkbox {
            name: self.name,
            value: Some(value),
            form: self.form,
        }
    }

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
}
impl<N: AttrValue, V: AttrValue, F: AttrValue> RunTimeHtmlSizeHint for Checkbox<N, V, F> {
    fn size_hint(&self) -> usize {
        self.name.size_hint() + self.value.size_hint() + self.form.size_hint()
    }
}
