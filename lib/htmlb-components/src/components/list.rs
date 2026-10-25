use crate::prelude::Unset;
use htmlb::prelude::*;

const LIST: &str = "md-list";

/// A vertical list of [`list_item`](crate::list_item)s.
pub fn list() -> List<Unset> {
    List { items: Unset }
}

pub struct List<I> {
    items: I,
}

impl<I> List<I> {
    pub fn items<I2: IntoHtml>(self, items: I2) -> List<I2> {
        List { items }
    }
}

impl<I: IntoHtml> Component for List<I> {
    fn render(self) -> impl IntoHtml {
        ul().class(LIST).child(self.items)
    }
}

impl<I: IntoHtml> RunTimeHtmlSizeHint for List<I> {
    fn size_hint(&self) -> usize {
        LIST.len() + self.items.size_hint()
    }
}
