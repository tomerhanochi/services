//! The components' stylesheet, `material.css`.

use crate::theme::fnv1a;

/// The components' styles. Colors, type and shapes are `var(--md-sys-...)` references to
/// the custom properties in [`Theme::css`](crate::Theme::css): serve both.
pub const CSS: &str = include_str!("material.css");

/// `material-<hash>.css`: changes whenever [`CSS`] does, so it can be served with
/// `Cache-Control: immutable`.
pub const CSS_FILE_NAME: &str = match std::str::from_utf8(&CSS_FILE_NAME_BYTES) {
    Ok(name) => name,
    Err(_) => panic!("file names are ASCII"),
};

const CSS_FILE_NAME_BYTES: [u8; 29] = hashed_file_name(fnv1a(CSS.as_bytes()));

/// `material-` + 16 hex digits + `.css`.
const fn hashed_file_name(hash: u64) -> [u8; 29] {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut name = *b"material-0000000000000000.css";
    let mut i = 0;
    while i < 16 {
        name[9 + i] = DIGITS[((hash >> (60 - 4 * i)) & 15) as usize];
        i += 1;
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class::{self, *};
    use crate::{ButtonStyle, ColorRole, Elevation, Shape, TypeScale};

    #[test]
    fn file_name_follows_content() {
        assert_eq!(
            CSS_FILE_NAME,
            format!("material-{:016x}.css", fnv1a(CSS.as_bytes()))
        );
    }

    #[test]
    fn every_class_is_styled() {
        let classes = [
            ICON,
            BUTTON,
            BUTTON_LABEL,
            ICON_BUTTON,
            TOP_APP_BAR,
            TOP_APP_BAR_NAVIGATION,
            TOP_APP_BAR_TITLES,
            TOP_APP_BAR_HEADLINE,
            TOP_APP_BAR_SUBTITLE,
            TOP_APP_BAR_ACTIONS,
            BOTTOM_APP_BAR,
            LIST,
            LIST_ITEM,
            LIST_ITEM_MAIN,
            LIST_ITEM_LEADING,
            LIST_ITEM_TEXT,
            LIST_ITEM_HEADLINE,
            LIST_ITEM_SUPPORTING,
            LIST_ITEM_TRAILING_TEXT,
            LIST_ITEM_TRAILING,
            SHEET,
            SHEET_HANDLE,
            SHEET_HEADLINE,
            MENU_ITEM,
            TEXT_FIELD,
            TEXT_FIELD_ERROR,
            TEXT_FIELD_INPUT,
            TEXT_FIELD_LABEL,
            TEXT_FIELD_SUPPORTING,
            SCAFFOLD,
            SCAFFOLD_BODY,
            PANE,
            SNACKBAR,
            DIVIDER,
        ];
        let buttons = ButtonStyle::ALL.iter().map(|&s| class::button(s));
        for name in classes.into_iter().chain(buttons) {
            // Followed by something that ends a class name, so `.md-list` doesn't count
            // as a rule for `md-list-item`.
            let styled = [" ", ",", ":", "{", "\n", "."]
                .iter()
                .any(|end| CSS.contains(&format!(".{name}{end}")));
            assert!(styled, "class `{name}` has no rule in material.css");
        }
    }

    /// The token classes are written out in `material.css`; this checks them against the
    /// tokens, so adding a token without its class fails here.
    #[test]
    fn token_classes_match_the_tokens() {
        let mut rules = Vec::new();
        for &scale in TypeScale::ALL {
            let n = scale.name();
            rules.push(format!(
                ".{} {{ font-family: var(--md-sys-typescale-{n}-font); font-weight: var(--md-sys-typescale-{n}-weight); \
                 font-size: var(--md-sys-typescale-{n}-size); line-height: var(--md-sys-typescale-{n}-line-height); \
                 letter-spacing: var(--md-sys-typescale-{n}-tracking); margin: 0; }}",
                class::type_scale(scale)
            ));
        }
        for &role in ColorRole::ALL {
            let n = role.name();
            rules.push(format!(
                ".{} {{ color: var(--md-sys-color-{n}); }}",
                class::text_color(role)
            ));
            let content = role
                .content()
                .map(|c| format!(" color: var(--md-sys-color-{});", c.name()))
                .unwrap_or_default();
            rules.push(format!(
                ".{} {{ background: var(--md-sys-color-{n});{content} }}",
                class::surface(role)
            ));
        }
        for &shape in Shape::ALL {
            rules.push(format!(
                ".{} {{ border-radius: var(--md-sys-shape-corner-{}); }}",
                class::shape(shape),
                shape.name()
            ));
        }
        for &level in Elevation::ALL {
            rules.push(format!(
                ".{} {{ box-shadow: var(--md-sys-elevation-shadow-{}); }}",
                class::elevation(level),
                level.name()
            ));
        }
        for rule in rules {
            assert!(
                CSS.lines().any(|line| line == rule),
                "material.css lacks\n{rule}"
            );
        }
    }

    #[test]
    fn braces_and_comments_are_balanced() {
        assert_eq!(CSS.matches('{').count(), CSS.matches('}').count());
        assert_eq!(CSS.matches("/*").count(), CSS.matches("*/").count());
    }
}
