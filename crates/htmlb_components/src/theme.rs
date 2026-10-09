//! A theme: the schemes and typefaces an app chooses, and its token stylesheet.

use std::fmt::Write;
use std::sync::OnceLock;

use crate::color::Scheme;
use crate::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY};
use crate::{Duration, Easing, Elevation, Shape, StateLayer, TypeScale, Typeface, Typefaces};

/// Fallbacks after the theme's typefaces. No web font is downloaded: if the theme's font
/// isn't installed (Roboto usually isn't on iOS or macOS), the platform's UI font is used.
const FALLBACK_FONTS: &str = "system-ui, -apple-system, \"Segoe UI\", sans-serif";

/// Everything that varies between apps: the color schemes and typefaces. Type scale,
/// shape, elevation, state and motion are fixed by MD3.
///
/// This crate defines no theme of its own; `htmlb_components_baseline` has
/// MD3's baseline. Declare a theme as a `static` so its stylesheet is generated once and
/// lives forever.
#[derive(Debug)]
pub struct Theme {
    pub light: Scheme,
    pub dark: Scheme,
    pub typefaces: Typefaces,
    css: OnceLock<String>,
}

impl Theme {
    pub const fn new(light: Scheme, dark: Scheme, typefaces: Typefaces) -> Self {
        Theme {
            light,
            dark,
            typefaces,
            css: OnceLock::new(),
        }
    }

    /// Every system token as a CSS custom property (`--md-sys-color-primary`, ...), with
    /// the dark scheme under `prefers-color-scheme: dark`. Generated on first use.
    pub fn css(&self) -> &str {
        self.css.get_or_init(|| tokens_css(self))
    }

    /// `theme-<hash>.css`: changes whenever [`css`](Self::css) does, so the stylesheet can
    /// be served with `Cache-Control: immutable`.
    pub fn css_file_name(&self) -> String {
        format!("theme-{:016x}.css", fnv1a(self.css().as_bytes()))
    }
}

impl Clone for Theme {
    fn clone(&self) -> Self {
        Theme::new(self.light.clone(), self.dark.clone(), self.typefaces)
    }
}

fn tokens_css(theme: &Theme) -> String {
    let mut css = String::with_capacity(12 * 1024);
    let scheme = |css: &mut String, scheme: &Scheme| {
        for (role, color) in scheme.iter() {
            let _ = write!(css, "  --md-sys-color-{}: ", role.name());
            color.write_hex(css);
            css.push_str(";\n");
        }
    };
    css.push_str(":root {\n  color-scheme: light dark;\n");
    scheme(&mut css, &theme.light);
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
    css.push_str("}\n@media (prefers-color-scheme: dark) {\n:root {\n");
    scheme(&mut css, &theme.dark);
    css.push_str("}\n}\n");
    css
}

/// FNV-1a, 64-bit: stable across builds and platforms, unlike `DefaultHasher`.
pub(crate) const fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash = (hash ^ bytes[i] as u64).wrapping_mul(0x0100_0000_01b3);
        i += 1;
    }
    hash
}
