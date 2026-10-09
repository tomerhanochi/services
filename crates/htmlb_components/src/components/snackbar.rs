use htmlb::prelude::*;

use crate::class::SNACKBAR;

/// A short message about what just happened, shown at the bottom and then dismissed.
pub fn snackbar<M: IntoHtml>(message: M) -> impl IntoHtml {
    div().class(SNACKBAR).role("status").child(message)
}
