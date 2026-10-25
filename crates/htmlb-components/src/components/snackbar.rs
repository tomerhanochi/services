use htmlb::prelude::*;

const SNACKBAR: &str = "md-snackbar";

/// A short message about what just happened, shown at the bottom and then dismissed.
pub fn snackbar<M: IntoHtml>(message: M) -> impl IntoHtml {
    div().class(SNACKBAR).role("status").child(message)
}
