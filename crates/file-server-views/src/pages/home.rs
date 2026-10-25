use file_server_core::prelude::MountName;
use htmlb::prelude::*;
use htmlb_components::prelude::*;

use crate::components::{empty, mount_row, sign_out};
use crate::page::page;

/// The mounts the user can see. `user` is the signed-in user's name, when they can sign out.
#[hotpath::measure]
pub fn home(user: Option<&str>, mounts: &[MountName]) -> impl IntoHtml {
    let body = match mounts.is_empty() {
        true => Either::Left(empty(
            "No folders are shared with you yet. Ask the server's administrator to give one of your roles access.",
        )),
        false => Either::Right(list(mounts.iter().map(mount_row).into_html_cloned())),
    };
    page()
        .title("Home")
        .content(scaffold(top_app_bar("Files").actions(sign_out(user)), body))
}
