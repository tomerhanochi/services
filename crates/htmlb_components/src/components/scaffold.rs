use htmlb::Component;
use htmlb::prelude::*;

use crate::class::{SCAFFOLD, SCAFFOLD_BODY};

/// A screen: a top app bar, the body, and optionally a bottom app bar and a snackbar.
pub fn scaffold<T: IntoHtml, B: IntoHtml>(top_app_bar: T, body: B) -> Scaffold<T, B, (), ()> {
    Scaffold {
        top_app_bar,
        body,
        bottom_app_bar: None,
        snackbar: None,
    }
}

pub struct Scaffold<T, B, F, S> {
    top_app_bar: T,
    body: B,
    bottom_app_bar: Option<F>,
    snackbar: Option<S>,
}

impl<T, B, F, S> Scaffold<T, B, F, S> {
    /// See [`bottom_app_bar`](crate::bottom_app_bar).
    pub fn bottom_app_bar<F2: IntoHtml>(self, bar: F2) -> Scaffold<T, B, F2, S> {
        Scaffold {
            top_app_bar: self.top_app_bar,
            body: self.body,
            bottom_app_bar: Some(bar),
            snackbar: self.snackbar,
        }
    }

    /// See [`snackbar`](crate::snackbar). Takes `Option` so a message can be shown only sometimes.
    pub fn snackbar<S2: IntoHtml>(self, snackbar: Option<S2>) -> Scaffold<T, B, F, S2> {
        Scaffold {
            top_app_bar: self.top_app_bar,
            body: self.body,
            bottom_app_bar: self.bottom_app_bar,
            snackbar,
        }
    }
}

impl<T: IntoHtml, B: IntoHtml, F: IntoHtml, S: IntoHtml> Component for Scaffold<T, B, F, S> {
    fn render(self) -> impl IntoHtml {
        div().class(SCAFFOLD).child((
            self.top_app_bar,
            main().class(SCAFFOLD_BODY).child(self.body),
            self.bottom_app_bar,
            self.snackbar.map(super::snackbar),
        ))
    }
    fn dynamic_len_hint(&self) -> usize {
        self.top_app_bar.len_hint()
            + self.body.len_hint()
            + self.bottom_app_bar.as_ref().map_or(0, F::len_hint)
            + self.snackbar.as_ref().map_or(0, S::len_hint)
    }
}
