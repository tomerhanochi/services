use htmlb::Component;
use htmlb::prelude::*;

use crate::{ColorRole, TypeScale, class};

/// Text in a [`TypeScale`] role: `<span>`, or `<p>` with [`paragraph`](Text::paragraph).
pub fn text<C: IntoHtml>(scale: TypeScale, content: C) -> Text<C> {
    Text {
        scale,
        content,
        color: None,
        paragraph: false,
    }
}

pub struct Text<C> {
    scale: TypeScale,
    content: C,
    color: Option<ColorRole>,
    paragraph: bool,
}

impl<C> Text<C> {
    /// Text color, e.g. [`ColorRole::OnSurfaceVariant`] for secondary text.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self
    }

    /// A block of its own (`<p>`) instead of inline text.
    pub fn paragraph(mut self) -> Self {
        self.paragraph = true;
        self
    }
}

impl<C: IntoHtml> Component for Text<C> {
    fn render(self) -> impl IntoHtml {
        let class = (
            class::type_scale(self.scale),
            self.color.map(class::text_color),
        );
        match self.paragraph {
            true => Either::Left(p().class(class).child(self.content)),
            false => Either::Right(span().class(class).child(self.content)),
        }
    }
    fn dynamic_len_hint(&self) -> usize {
        self.content.len_hint()
    }
}
