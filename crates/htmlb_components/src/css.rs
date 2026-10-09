//! CSS generation for a complete themed component stylesheet.

use std::fmt::Write;

use crate::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY};
use crate::{Duration, Easing, Elevation, Shape, StateLayer, Theme, TypeScale, Typeface};

const FALLBACK_FONTS: &str = "system-ui, -apple-system, \"Segoe UI\", sans-serif";

/// Returns a complete stylesheet: the theme tokens followed by the component rules.
pub fn css(theme: &Theme) -> String {
    let mut css = tokens_css(theme);
    css.push_str(crate::stylesheet::CSS);
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
    for (typeface, name) in [(Typeface::Brand, "brand"), (Typeface::Plain, "plain")] {
        let _ = writeln!(
            css,
            "  --md-ref-typeface-{name}: {}, {FALLBACK_FONTS};",
            theme.typefaces.get(typeface)
        );
    }
    for &scale in TypeScale::ALL {
        let (name, style) = (scale.name(), scale.style());
        let font = match style.typeface {
            Typeface::Brand => "var(--md-ref-typeface-brand)",
            Typeface::Plain => "var(--md-ref-typeface-plain)",
        };
        let _ = writeln!(css, "  --md-sys-typescale-{name}-font: {font};");
        let _ = writeln!(css, "  --md-sys-typescale-{name}-weight: {};", style.weight);
        let _ = writeln!(
            css,
            "  --md-sys-typescale-{name}-size: {}rem;",
            style.size_rem
        );
        let _ = writeln!(
            css,
            "  --md-sys-typescale-{name}-line-height: {}rem;",
            style.line_height_rem
        );
        let _ = writeln!(
            css,
            "  --md-sys-typescale-{name}-tracking: {}rem;",
            style.tracking_rem
        );
    }
    for &shape in Shape::ALL {
        let _ = writeln!(
            css,
            "  --md-sys-shape-corner-{}: {}px;",
            shape.name(),
            shape.radius_px()
        );
    }
    for &level in Elevation::ALL {
        let _ = writeln!(
            css,
            "  --md-sys-elevation-shadow-{}: {};",
            level.name(),
            level.shadow()
        );
    }
    for &state in StateLayer::ALL {
        let _ = writeln!(
            css,
            "  --md-sys-state-{}-state-layer-opacity: {};",
            state.name(),
            state.opacity()
        );
    }
    let _ = writeln!(
        css,
        "  --md-sys-state-disabled-content-opacity: {DISABLED_CONTENT_OPACITY};"
    );
    let _ = writeln!(
        css,
        "  --md-sys-state-disabled-container-opacity: {DISABLED_CONTAINER_OPACITY};"
    );
    for &easing in Easing::ALL {
        let [x1, y1, x2, y2] = easing.cubic_bezier();
        let _ = writeln!(
            css,
            "  --md-sys-motion-easing-{}: cubic-bezier({x1}, {y1}, {x2}, {y2});",
            easing.name()
        );
    }
    for &duration in Duration::ALL {
        let _ = writeln!(
            css,
            "  --md-sys-motion-duration-{}: {}ms;",
            duration.name(),
            duration.millis()
        );
    }
    css.push_str("}\n");
    css
}
