use crate::prelude::Unset;
use htmlb::prelude::*;

const SNACKBAR: &str = "md-snackbar";

/// A short message about what just happened, shown at the bottom and then dismissed.
pub fn snackbar() -> Snackbar<Unset> {
    Snackbar { message: Unset }
}

pub struct Snackbar<M> {
    message: M,
}

impl<M> Snackbar<M> {
    pub fn message<M2: IntoHtml>(self, message: M2) -> Snackbar<M2> {
        Snackbar { message }
    }
}

impl<M: IntoHtml> Component for Snackbar<M> {
    fn render(self) -> impl IntoHtml {
        div().class(SNACKBAR).role("status").child(self.message)
    }
}

impl<M: IntoHtml> RunTimeHtmlSizeHint for Snackbar<M> {
    fn size_hint(&self) -> usize {
        SNACKBAR.len() + "status".len() + self.message.size_hint()
    }
}
