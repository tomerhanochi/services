use crate::prelude::Unset;
use htmlb::prelude::*;

const SCAFFOLD: &str = "md-scaffold";
const SCAFFOLD_BODY: &str = "md-scaffold__body";

/// A screen: a top app bar, the body, and optionally a bottom app bar and a snackbar.
pub fn scaffold() -> Scaffold<Unset, Unset, (), ()> {
    Scaffold {
        top_app_bar: Unset,
        body: Unset,
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
    pub fn top_app_bar<T2: IntoHtml>(self, top_app_bar: T2) -> Scaffold<T2, B, F, S> {
        Scaffold {
            top_app_bar,
            body: self.body,
            bottom_app_bar: self.bottom_app_bar,
            snackbar: self.snackbar,
        }
    }

    pub fn body<B2: IntoHtml>(self, body: B2) -> Scaffold<T, B2, F, S> {
        Scaffold {
            top_app_bar: self.top_app_bar,
            body,
            bottom_app_bar: self.bottom_app_bar,
            snackbar: self.snackbar,
        }
    }

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
            self.snackbar
                .map(|message| super::snackbar().message(message)),
        ))
    }
}
impl<T: IntoHtml, B: IntoHtml, F: IntoHtml, S: IntoHtml> RunTimeHtmlSizeHint
    for Scaffold<T, B, F, S>
{
    fn size_hint(&self) -> usize {
        self.top_app_bar.size_hint()
            + self.body.size_hint()
            + self.bottom_app_bar.size_hint()
            + self.snackbar.size_hint()
    }
}
