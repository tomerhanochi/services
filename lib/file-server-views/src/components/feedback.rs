use htmlb::prelude::*;
use htmlb_components::prelude::*;

/// What just happened, shown as a snackbar after a redirect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    Renamed,
    Moved,
}

impl Done {
    pub const fn query(self) -> &'static str {
        match self {
            Done::Renamed => "renamed",
            Done::Moved => "moved",
        }
    }

    pub fn from_query(s: &str) -> Option<Done> {
        match s {
            "renamed" => Some(Done::Renamed),
            "moved" => Some(Done::Moved),
            _ => None,
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Done::Renamed => "Renamed",
            Done::Moved => "Moved",
        }
    }
}

/// In place of a list with nothing in it, or of a page that couldn't be shown.
pub fn empty(message: &str) -> impl IntoHtml {
    pane().child(supporting_text().child(message))
}

/// Why a form's submission failed, above the form.
pub fn error_message(message: &str) -> impl IntoHtml {
    pane().child(p().class("md-color-error").child(message))
}
