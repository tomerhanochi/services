use htmlb::Component;
use htmlb::prelude::*;

use crate::{ColorRole, class};

/// A container filled with a color role; its text takes the matching content color.
pub fn surface(role: ColorRole) -> Surface<()> {
    Surface { role, content: () }
}

pub struct Surface<C> {
    role: ColorRole,
    content: C,
}

impl<C> Surface<C> {
    pub fn child<N: IntoHtml>(self, child: N) -> Surface<(C, N)> {
        Surface {
            role: self.role,
            content: (self.content, child),
        }
    }
}

impl<C: IntoHtml> Component for Surface<C> {
    fn render(self) -> impl IntoHtml {
        div()
            .class((("md-surface", class::surface(self.role)),))
            .child(self.content)
    }
    fn dynamic_len_hint(&self) -> usize {
        self.content.len_hint()
    }
}
