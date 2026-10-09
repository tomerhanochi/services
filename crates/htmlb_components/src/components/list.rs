use htmlb::prelude::*;

use crate::class::LIST;

/// A vertical list of [`list_item`](crate::list_item)s.
pub fn list<I: IntoHtml>(items: I) -> impl IntoHtml {
    ul().class(LIST).child(items)
}
