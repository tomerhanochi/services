//! Material Symbols (Rounded, weight 400), inlined as SVG so pages need no icon font. Each
//! icon is a zero-sized type that writes its `icon/<name>.svg` file verbatim, so its length
//! is known exactly at compile time.
//!
//! Path data from the `@material-symbols/svg-400` npm package (Apache License 2.0). To add
//! an icon, copy its `<path d>` from that package's `rounded/<name>.svg` into a new file
//! here, shaped like the others (`class="md-icon"`, `aria-hidden`, no `width`/`height`),
//! and declare it below.

use htmlb::IntoHtml;

/// An icon. Rendered `aria-hidden`: pair it with a text label or `aria-label`.
pub trait Icon: IntoHtml + Copy {
    /// The complete `<svg>` element.
    const SVG: &'static str;
}

/// `icon!(Name, "file.svg")` declares `pub struct Name`, rendering `icon/file.svg`.
macro_rules! icon {
    ($name:ident, $file:literal) => {
        #[doc = concat!("`", $file, "`")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name;

        impl Icon for $name {
            const SVG: &'static str = include_str!(concat!("icon/", $file)).trim_ascii_end();
        }

        impl IntoHtml for $name {
            const MIN_LEN: usize = <Self as Icon>::SVG.len();
            #[inline]
            fn write_html(self, buf: &mut String) {
                buf.push_str(<Self as Icon>::SVG)
            }
        }
    };
}

icon!(ArrowBack, "arrow_back.svg");
icon!(ArrowUpward, "arrow_upward.svg");
icon!(Check, "check.svg");
icon!(ChevronRight, "chevron_right.svg");
icon!(Close, "close.svg");
icon!(Download, "download.svg");
icon!(Draft, "draft.svg");
icon!(DriveFileMove, "drive_file_move.svg");
icon!(Edit, "edit.svg");
icon!(Error, "error.svg");
icon!(Folder, "folder.svg");
icon!(HardDrive, "hard_drive.svg");
icon!(Home, "home.svg");
icon!(Lock, "lock.svg");
icon!(Login, "login.svg");
icon!(Logout, "logout.svg");
icon!(MoreVert, "more_vert.svg");

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: &[&str] = &[
        ArrowBack::SVG,
        ArrowUpward::SVG,
        Check::SVG,
        ChevronRight::SVG,
        Close::SVG,
        Download::SVG,
        Draft::SVG,
        DriveFileMove::SVG,
        Edit::SVG,
        Error::SVG,
        Folder::SVG,
        HardDrive::SVG,
        Home::SVG,
        Lock::SVG,
        Login::SVG,
        Logout::SVG,
        MoreVert::SVG,
    ];

    #[test]
    fn every_icon_is_one_hidden_svg_element() {
        for svg in ALL {
            assert!(svg.starts_with(r#"<svg class="md-icon" viewBox="0 -960 960 960" aria-hidden="true" focusable="false">"#), "{svg}");
            assert!(svg.ends_with("</svg>") && !svg.contains('\n'), "{svg}");
        }
    }

    #[test]
    fn length_is_exact() {
        assert_eq!(ArrowBack.len_hint(), ArrowBack.to_html().len());
        assert_eq!(<ArrowBack as IntoHtml>::MIN_LEN, ArrowBack::SVG.len());
    }
}
