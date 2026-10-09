//! CSS generation for a complete themed component stylesheet.

use std::fmt::Write;

use crate::Theme;

const CSS: &str = include_str!("material.css");

const FALLBACK_FONTS: &str = "system-ui, -apple-system, \"Segoe UI\", sans-serif";

/// Returns a complete stylesheet: the theme tokens followed by the component rules.
pub fn css(theme: &Theme) -> String {
    let mut css = tokens_css(theme);
    css.push_str(CSS);
    css
}

fn tokens_css(theme: &Theme) -> String {
    let mut css = String::with_capacity(12 * 1024);
    let colors = |css: &mut String| {
        for (role, light) in theme.light.iter() {
            let dark = theme.dark.get(role);
            let _ = write!(css, "  --md-sys-color-{}: light-dark(", role.name());
            light.write_hex(css);
            css.push_str(", ");
            dark.write_hex(css);
            css.push_str(");\n");
        }
    };
    css.push_str(":root {\n  color-scheme: light dark;\n");
    colors(&mut css);
    for (name, font) in [
        ("brand", theme.typefaces.brand),
        ("plain", theme.typefaces.plain),
    ] {
        let _ = writeln!(css, "  --md-ref-typeface-{name}: {font}, {FALLBACK_FONTS};");
    }
    css.push_str("}\n");
    css
}
