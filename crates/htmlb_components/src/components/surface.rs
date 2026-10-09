use htmlb::Component;
use htmlb::prelude::*;

use crate::{ColorRole, Elevation, Shape, class};

/// A container filled with a color role; its text takes the matching content color.
pub fn surface(role: ColorRole) -> Surface<()> {
    Surface {
        role,
        shape: None,
        elevation: None,
        content: (),
    }
}

pub struct Surface<C> {
    role: ColorRole,
    shape: Option<Shape>,
    elevation: Option<Elevation>,
    content: C,
}

impl<C> Surface<C> {
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = Some(elevation);
        self
    }

    pub fn child<N: IntoHtml>(self, child: N) -> Surface<(C, N)> {
        Surface {
            role: self.role,
            shape: self.shape,
            elevation: self.elevation,
            content: (self.content, child),
        }
    }
}

impl<C: IntoHtml> Component for Surface<C> {
    fn render(self) -> impl IntoHtml {
        div()
            .class((
                class::surface(self.role),
                self.shape.map(class::shape),
                self.elevation.map(class::elevation),
            ))
            .child(self.content)
    }
    fn dynamic_len_hint(&self) -> usize {
        self.content.len_hint()
    }
}
