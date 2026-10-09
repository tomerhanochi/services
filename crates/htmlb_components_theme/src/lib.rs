//! MD3's baseline theme for [`htmlb_components`]: the purple `#6750a4` schemes
//! and Roboto, from `@material/web` 2.5.0's token files (`tokens/versions/v0_192`).
//!
//! ```
//! use htmlb_components_baseline::THEME;
//!
//! let css = htmlb_components_theme::css();
//! assert!(css.contains("--md-sys-color-primary: light-dark(#6750a4, #d0bcff);"));
//! ```
//!
//! To change a few roles, start from a baseline scheme:
//!
//! ```
//! use htmlb_components::{Color, Scheme, Theme};
//! use htmlb_components_baseline::{DARK, LIGHT, TYPEFACES};
//!
//! static THEME: Theme = Theme::new(
//!     Scheme { primary: Color::hex(0x006a6a), ..LIGHT },
//!     DARK,
//!     TYPEFACES,
//! );
//! assert!(htmlb_components::css::css(&THEME).contains("--md-sys-color-primary: light-dark(#006a6a,"));
//! ```

use htmlb_components::{Color, Scheme, Theme, Typefaces};
use std::sync::OnceLock;

static CSS: OnceLock<String> = OnceLock::new();

/// The full baseline theme and component stylesheet, rendered once and retained here.
pub fn css() -> &'static str {
    CSS.get_or_init(|| htmlb_components::css::css(&THEME))
}

/// The baseline theme: [`LIGHT`], [`DARK`] and [`TYPEFACES`].
pub static THEME: Theme = Theme::new(LIGHT, DARK, TYPEFACES);

pub const TYPEFACES: Typefaces = Typefaces {
    brand: "Roboto",
    plain: "Roboto",
};

pub const LIGHT: Scheme = Scheme {
    primary: Color::hex(0x6750a4),
    on_primary: Color::hex(0xffffff),
    primary_container: Color::hex(0xeaddff),
    on_primary_container: Color::hex(0x21005d),
    primary_fixed: Color::hex(0xeaddff),
    primary_fixed_dim: Color::hex(0xd0bcff),
    on_primary_fixed: Color::hex(0x21005d),
    on_primary_fixed_variant: Color::hex(0x4f378b),
    inverse_primary: Color::hex(0xd0bcff),
    secondary: Color::hex(0x625b71),
    on_secondary: Color::hex(0xffffff),
    secondary_container: Color::hex(0xe8def8),
    on_secondary_container: Color::hex(0x1d192b),
    secondary_fixed: Color::hex(0xe8def8),
    secondary_fixed_dim: Color::hex(0xccc2dc),
    on_secondary_fixed: Color::hex(0x1d192b),
    on_secondary_fixed_variant: Color::hex(0x4a4458),
    tertiary: Color::hex(0x7d5260),
    on_tertiary: Color::hex(0xffffff),
    tertiary_container: Color::hex(0xffd8e4),
    on_tertiary_container: Color::hex(0x31111d),
    tertiary_fixed: Color::hex(0xffd8e4),
    tertiary_fixed_dim: Color::hex(0xefb8c8),
    on_tertiary_fixed: Color::hex(0x31111d),
    on_tertiary_fixed_variant: Color::hex(0x633b48),
    error: Color::hex(0xb3261e),
    on_error: Color::hex(0xffffff),
    error_container: Color::hex(0xf9dedc),
    on_error_container: Color::hex(0x410e0b),
    surface: Color::hex(0xfef7ff),
    on_surface: Color::hex(0x1d1b20),
    surface_variant: Color::hex(0xe7e0ec),
    on_surface_variant: Color::hex(0x49454f),
    surface_dim: Color::hex(0xded8e1),
    surface_bright: Color::hex(0xfef7ff),
    surface_container_lowest: Color::hex(0xffffff),
    surface_container_low: Color::hex(0xf7f2fa),
    surface_container: Color::hex(0xf3edf7),
    surface_container_high: Color::hex(0xece6f0),
    surface_container_highest: Color::hex(0xe6e0e9),
    surface_tint: Color::hex(0x6750a4),
    inverse_surface: Color::hex(0x322f35),
    inverse_on_surface: Color::hex(0xf5eff7),
    background: Color::hex(0xfef7ff),
    on_background: Color::hex(0x1d1b20),
    outline: Color::hex(0x79747e),
    outline_variant: Color::hex(0xcac4d0),
    shadow: Color::hex(0x000000),
    scrim: Color::hex(0x000000),
};

pub const DARK: Scheme = Scheme {
    primary: Color::hex(0xd0bcff),
    on_primary: Color::hex(0x381e72),
    primary_container: Color::hex(0x4f378b),
    on_primary_container: Color::hex(0xeaddff),
    primary_fixed: Color::hex(0xeaddff),
    primary_fixed_dim: Color::hex(0xd0bcff),
    on_primary_fixed: Color::hex(0x21005d),
    on_primary_fixed_variant: Color::hex(0x4f378b),
    inverse_primary: Color::hex(0x6750a4),
    secondary: Color::hex(0xccc2dc),
    on_secondary: Color::hex(0x332d41),
    secondary_container: Color::hex(0x4a4458),
    on_secondary_container: Color::hex(0xe8def8),
    secondary_fixed: Color::hex(0xe8def8),
    secondary_fixed_dim: Color::hex(0xccc2dc),
    on_secondary_fixed: Color::hex(0x1d192b),
    on_secondary_fixed_variant: Color::hex(0x4a4458),
    tertiary: Color::hex(0xefb8c8),
    on_tertiary: Color::hex(0x492532),
    tertiary_container: Color::hex(0x633b48),
    on_tertiary_container: Color::hex(0xffd8e4),
    tertiary_fixed: Color::hex(0xffd8e4),
    tertiary_fixed_dim: Color::hex(0xefb8c8),
    on_tertiary_fixed: Color::hex(0x31111d),
    on_tertiary_fixed_variant: Color::hex(0x633b48),
    error: Color::hex(0xf2b8b5),
    on_error: Color::hex(0x601410),
    error_container: Color::hex(0x8c1d18),
    on_error_container: Color::hex(0xf9dedc),
    surface: Color::hex(0x141218),
    on_surface: Color::hex(0xe6e0e9),
    surface_variant: Color::hex(0x49454f),
    on_surface_variant: Color::hex(0xcac4d0),
    surface_dim: Color::hex(0x141218),
    surface_bright: Color::hex(0x3b383e),
    surface_container_lowest: Color::hex(0x0f0d13),
    surface_container_low: Color::hex(0x1d1b20),
    surface_container: Color::hex(0x211f26),
    surface_container_high: Color::hex(0x2b2930),
    surface_container_highest: Color::hex(0x36343b),
    surface_tint: Color::hex(0xd0bcff),
    inverse_surface: Color::hex(0xe6e0e9),
    inverse_on_surface: Color::hex(0x322f35),
    background: Color::hex(0x141218),
    on_background: Color::hex(0xe6e0e9),
    outline: Color::hex(0x938f99),
    outline_variant: Color::hex(0x49454f),
    shadow: Color::hex(0x000000),
    scrim: Color::hex(0x000000),
};

#[cfg(test)]
mod tests {
    use super::*;
    use htmlb::IntoHtml;
    use htmlb_components::{CSS, ColorRole};

    #[test]
    fn roles_get_their_own_color() {
        assert_eq!(LIGHT.get(ColorRole::Primary).to_html(), "#6750a4");
        assert_eq!(LIGHT.get(ColorRole::Scrim).to_html(), "#000000");
        assert_eq!(DARK.get(ColorRole::OnSurface).to_html(), "#e6e0e9");
        for (i, role) in ColorRole::ALL.iter().enumerate() {
            assert_eq!(*role as usize, i);
        }
    }

    #[test]
    fn css_defines_every_role_in_both_schemes() {
        let css = css();
        for &role in ColorRole::ALL {
            let property = format!("--md-sys-color-{}:", role.name());
            assert_eq!(css.matches(&property).count(), 1, "{role:?}");
        }
        assert_eq!(css.matches('{').count(), css.matches('}').count());
    }

    #[test]
    /// Every `var()` in the components' stylesheet is defined by it or by the theme's.
    #[test]
    fn every_variable_the_components_use_is_defined() {
        let css = without_comments(CSS);
        let defined = format!("{}{css}", htmlb_components::css::css(&THEME));
        let mut rest = css.as_str();
        while let Some(at) = rest.find("var(--") {
            rest = &rest[at + 4..];
            let name = &rest[..rest.find(')').unwrap()];
            assert!(
                defined.contains(&format!("{name}:")),
                "{name} is used but never defined"
            );
        }
    }

    fn without_comments(css: &str) -> String {
        let mut out = String::new();
        let mut rest = css;
        while let Some(start) = rest.find("/*") {
            out.push_str(&rest[..start]);
            rest = &rest[start + rest[start..].find("*/").unwrap() + 2..];
        }
        out + rest
    }
}
