use file_server_core::prelude::Location;
use htmlb::AttrValue;
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::icons::Icons;
use crate::urls::{self, ParentHref, SelectHref};

/// A top app bar's navigation leaving a full-screen dialog for `href`.
pub fn close_button<L: AttrValue + Clone, H: AttrValue>(name: L, href: H) -> impl IntoHtml {
    icon_button().icon(Icons::Close).label(name).href(href)
}

/// A top app bar's navigation to the list of mounts.
pub fn home_button() -> impl IntoHtml {
    icon_button()
        .icon(Icons::Home)
        .label("All folders")
        .href(urls::HOME)
}

/// A listing's navigation: to the folder above `at`, or to the mounts from a mount's root.
pub fn up_button(at: &Location) -> impl IntoHtml {
    match at.parent() {
        Some(parent) => Either::Left(
            icon_button()
                .icon(Icons::ArrowBack)
                .label("Up one folder")
                .href(ParentHref(parent)),
        ),
        None => Either::Right(
            icon_button()
                .icon(Icons::ArrowBack)
                .label("All folders")
                .href(urls::HOME),
        ),
    }
}

/// Opens `at`'s listing in selection mode.
pub fn select_button(at: &Location) -> impl IntoHtml {
    icon_button()
        .icon(Icons::Checklist)
        .label("Select")
        .href(SelectHref(at))
}

/// Sign out, as a form: logging out changes state, so it's a POST. `user` is the
/// signed-in user's name, when they can sign out.
pub fn sign_out(user: Option<&str>) -> impl IntoHtml {
    user.map(|name| {
        form().method(FormMethod::Post).action(urls::LOGOUT).child(
            icon_button()
                .icon(Icons::Logout)
                .label(("Sign out ", name))
                .submit(),
        )
    })
}
