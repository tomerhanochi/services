//! Color roles (`md.sys.color.*`) and schemes that assign them colors.

use std::fmt;

use htmlb::{AttrValue, IntoHtml};

/// An opaque sRGB color.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// From `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Color {
            r: (rgb >> 16) as u8,
            g: (rgb >> 8) as u8,
            b: rgb as u8,
        }
    }
}

impl Color {
    /// `#rrggbb`, without allocating.
    pub const fn to_hex(self) -> [u8; 7] {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let Color { r, g, b } = self;
        [
            b'#',
            DIGITS[(r >> 4) as usize],
            DIGITS[(r & 15) as usize],
            DIGITS[(g >> 4) as usize],
            DIGITS[(g & 15) as usize],
            DIGITS[(b >> 4) as usize],
            DIGITS[(b & 15) as usize],
        ]
    }

    /// Appends `#rrggbb` to `buf`.
    pub fn write_hex(self, buf: &mut String) {
        let hex = self.to_hex();
        // Only ASCII digits and `#`.
        buf.push_str(std::str::from_utf8(&hex).expect("hex digits are ASCII"));
    }
}

/// `#rrggbb`, e.g. for `<meta name="theme-color" content>`.
impl AttrValue for Color {
    const VALUE_MIN_LEN: usize = 7;
    fn value_len_hint(&self) -> usize {
        7
    }
    fn write_value(self, buf: &mut String) {
        self.write_hex(buf)
    }
}

/// `#rrggbb` as text.
impl IntoHtml for Color {
    const MIN_LEN: usize = 7;
    fn write_html(self, buf: &mut String) {
        self.write_hex(buf)
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = self.to_hex();
        f.write_str(std::str::from_utf8(&hex).expect("hex digits are ASCII"))
    }
}

/// What a color is *for*. Each `on-*` role is the content color for the role it names,
/// with guaranteed contrast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorRole {
    Primary,
    OnPrimary,
    PrimaryContainer,
    OnPrimaryContainer,
    PrimaryFixed,
    PrimaryFixedDim,
    OnPrimaryFixed,
    OnPrimaryFixedVariant,
    InversePrimary,
    Secondary,
    OnSecondary,
    SecondaryContainer,
    OnSecondaryContainer,
    SecondaryFixed,
    SecondaryFixedDim,
    OnSecondaryFixed,
    OnSecondaryFixedVariant,
    Tertiary,
    OnTertiary,
    TertiaryContainer,
    OnTertiaryContainer,
    TertiaryFixed,
    TertiaryFixedDim,
    OnTertiaryFixed,
    OnTertiaryFixedVariant,
    Error,
    OnError,
    ErrorContainer,
    OnErrorContainer,
    Surface,
    OnSurface,
    SurfaceVariant,
    OnSurfaceVariant,
    SurfaceDim,
    SurfaceBright,
    SurfaceContainerLowest,
    SurfaceContainerLow,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    SurfaceTint,
    InverseSurface,
    InverseOnSurface,
    Background,
    OnBackground,
    Outline,
    OutlineVariant,
    Shadow,
    Scrim,
}

impl ColorRole {
    pub const ALL: &[ColorRole] = &[
        ColorRole::Primary,
        ColorRole::OnPrimary,
        ColorRole::PrimaryContainer,
        ColorRole::OnPrimaryContainer,
        ColorRole::PrimaryFixed,
        ColorRole::PrimaryFixedDim,
        ColorRole::OnPrimaryFixed,
        ColorRole::OnPrimaryFixedVariant,
        ColorRole::InversePrimary,
        ColorRole::Secondary,
        ColorRole::OnSecondary,
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
        ColorRole::SecondaryFixed,
        ColorRole::SecondaryFixedDim,
        ColorRole::OnSecondaryFixed,
        ColorRole::OnSecondaryFixedVariant,
        ColorRole::Tertiary,
        ColorRole::OnTertiary,
        ColorRole::TertiaryContainer,
        ColorRole::OnTertiaryContainer,
        ColorRole::TertiaryFixed,
        ColorRole::TertiaryFixedDim,
        ColorRole::OnTertiaryFixed,
        ColorRole::OnTertiaryFixedVariant,
        ColorRole::Error,
        ColorRole::OnError,
        ColorRole::ErrorContainer,
        ColorRole::OnErrorContainer,
        ColorRole::Surface,
        ColorRole::OnSurface,
        ColorRole::SurfaceVariant,
        ColorRole::OnSurfaceVariant,
        ColorRole::SurfaceDim,
        ColorRole::SurfaceBright,
        ColorRole::SurfaceContainerLowest,
        ColorRole::SurfaceContainerLow,
        ColorRole::SurfaceContainer,
        ColorRole::SurfaceContainerHigh,
        ColorRole::SurfaceContainerHighest,
        ColorRole::SurfaceTint,
        ColorRole::InverseSurface,
        ColorRole::InverseOnSurface,
        ColorRole::Background,
        ColorRole::OnBackground,
        ColorRole::Outline,
        ColorRole::OutlineVariant,
        ColorRole::Shadow,
        ColorRole::Scrim,
    ];

    /// The token's name in MD3, e.g. `on-primary-container`.
    pub const fn name(self) -> &'static str {
        match self {
            ColorRole::Primary => "primary",
            ColorRole::OnPrimary => "on-primary",
            ColorRole::PrimaryContainer => "primary-container",
            ColorRole::OnPrimaryContainer => "on-primary-container",
            ColorRole::PrimaryFixed => "primary-fixed",
            ColorRole::PrimaryFixedDim => "primary-fixed-dim",
            ColorRole::OnPrimaryFixed => "on-primary-fixed",
            ColorRole::OnPrimaryFixedVariant => "on-primary-fixed-variant",
            ColorRole::InversePrimary => "inverse-primary",
            ColorRole::Secondary => "secondary",
            ColorRole::OnSecondary => "on-secondary",
            ColorRole::SecondaryContainer => "secondary-container",
            ColorRole::OnSecondaryContainer => "on-secondary-container",
            ColorRole::SecondaryFixed => "secondary-fixed",
            ColorRole::SecondaryFixedDim => "secondary-fixed-dim",
            ColorRole::OnSecondaryFixed => "on-secondary-fixed",
            ColorRole::OnSecondaryFixedVariant => "on-secondary-fixed-variant",
            ColorRole::Tertiary => "tertiary",
            ColorRole::OnTertiary => "on-tertiary",
            ColorRole::TertiaryContainer => "tertiary-container",
            ColorRole::OnTertiaryContainer => "on-tertiary-container",
            ColorRole::TertiaryFixed => "tertiary-fixed",
            ColorRole::TertiaryFixedDim => "tertiary-fixed-dim",
            ColorRole::OnTertiaryFixed => "on-tertiary-fixed",
            ColorRole::OnTertiaryFixedVariant => "on-tertiary-fixed-variant",
            ColorRole::Error => "error",
            ColorRole::OnError => "on-error",
            ColorRole::ErrorContainer => "error-container",
            ColorRole::OnErrorContainer => "on-error-container",
            ColorRole::Surface => "surface",
            ColorRole::OnSurface => "on-surface",
            ColorRole::SurfaceVariant => "surface-variant",
            ColorRole::OnSurfaceVariant => "on-surface-variant",
            ColorRole::SurfaceDim => "surface-dim",
            ColorRole::SurfaceBright => "surface-bright",
            ColorRole::SurfaceContainerLowest => "surface-container-lowest",
            ColorRole::SurfaceContainerLow => "surface-container-low",
            ColorRole::SurfaceContainer => "surface-container",
            ColorRole::SurfaceContainerHigh => "surface-container-high",
            ColorRole::SurfaceContainerHighest => "surface-container-highest",
            ColorRole::SurfaceTint => "surface-tint",
            ColorRole::InverseSurface => "inverse-surface",
            ColorRole::InverseOnSurface => "inverse-on-surface",
            ColorRole::Background => "background",
            ColorRole::OnBackground => "on-background",
            ColorRole::Outline => "outline",
            ColorRole::OutlineVariant => "outline-variant",
            ColorRole::Shadow => "shadow",
            ColorRole::Scrim => "scrim",
        }
    }

    /// The role for text and icons drawn on this one, e.g. `OnPrimaryContainer` for
    /// `PrimaryContainer`. `None` for roles that aren't backgrounds (`OnPrimary`, `Outline`, ...).
    pub const fn content(self) -> Option<ColorRole> {
        use ColorRole::*;
        Some(match self {
            Primary => OnPrimary,
            PrimaryContainer => OnPrimaryContainer,
            PrimaryFixed | PrimaryFixedDim => OnPrimaryFixed,
            Secondary => OnSecondary,
            SecondaryContainer => OnSecondaryContainer,
            SecondaryFixed | SecondaryFixedDim => OnSecondaryFixed,
            Tertiary => OnTertiary,
            TertiaryContainer => OnTertiaryContainer,
            TertiaryFixed | TertiaryFixedDim => OnTertiaryFixed,
            Error => OnError,
            ErrorContainer => OnErrorContainer,
            Surface
            | SurfaceDim
            | SurfaceBright
            | SurfaceContainerLowest
            | SurfaceContainerLow
            | SurfaceContainer
            | SurfaceContainerHigh
            | SurfaceContainerHighest => OnSurface,
            SurfaceVariant => OnSurfaceVariant,
            InverseSurface => InverseOnSurface,
            Background => OnBackground,
            _ => return None,
        })
    }
}

/// A color for every [`ColorRole`]: a struct literal must name them all.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Scheme {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub on_primary_fixed: Color,
    pub on_primary_fixed_variant: Color,
    pub inverse_primary: Color,
    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub on_secondary_fixed: Color,
    pub on_secondary_fixed_variant: Color,
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_tertiary_fixed: Color,
    pub on_tertiary_fixed_variant: Color,
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
    pub surface: Color,
    pub on_surface: Color,
    pub surface_variant: Color,
    pub on_surface_variant: Color,
    pub surface_dim: Color,
    pub surface_bright: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub surface_tint: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub background: Color,
    pub on_background: Color,
    pub outline: Color,
    pub outline_variant: Color,
    pub shadow: Color,
    pub scrim: Color,
}

impl Scheme {
    pub const fn get(&self, role: ColorRole) -> Color {
        match role {
            ColorRole::Primary => self.primary,
            ColorRole::OnPrimary => self.on_primary,
            ColorRole::PrimaryContainer => self.primary_container,
            ColorRole::OnPrimaryContainer => self.on_primary_container,
            ColorRole::PrimaryFixed => self.primary_fixed,
            ColorRole::PrimaryFixedDim => self.primary_fixed_dim,
            ColorRole::OnPrimaryFixed => self.on_primary_fixed,
            ColorRole::OnPrimaryFixedVariant => self.on_primary_fixed_variant,
            ColorRole::InversePrimary => self.inverse_primary,
            ColorRole::Secondary => self.secondary,
            ColorRole::OnSecondary => self.on_secondary,
            ColorRole::SecondaryContainer => self.secondary_container,
            ColorRole::OnSecondaryContainer => self.on_secondary_container,
            ColorRole::SecondaryFixed => self.secondary_fixed,
            ColorRole::SecondaryFixedDim => self.secondary_fixed_dim,
            ColorRole::OnSecondaryFixed => self.on_secondary_fixed,
            ColorRole::OnSecondaryFixedVariant => self.on_secondary_fixed_variant,
            ColorRole::Tertiary => self.tertiary,
            ColorRole::OnTertiary => self.on_tertiary,
            ColorRole::TertiaryContainer => self.tertiary_container,
            ColorRole::OnTertiaryContainer => self.on_tertiary_container,
            ColorRole::TertiaryFixed => self.tertiary_fixed,
            ColorRole::TertiaryFixedDim => self.tertiary_fixed_dim,
            ColorRole::OnTertiaryFixed => self.on_tertiary_fixed,
            ColorRole::OnTertiaryFixedVariant => self.on_tertiary_fixed_variant,
            ColorRole::Error => self.error,
            ColorRole::OnError => self.on_error,
            ColorRole::ErrorContainer => self.error_container,
            ColorRole::OnErrorContainer => self.on_error_container,
            ColorRole::Surface => self.surface,
            ColorRole::OnSurface => self.on_surface,
            ColorRole::SurfaceVariant => self.surface_variant,
            ColorRole::OnSurfaceVariant => self.on_surface_variant,
            ColorRole::SurfaceDim => self.surface_dim,
            ColorRole::SurfaceBright => self.surface_bright,
            ColorRole::SurfaceContainerLowest => self.surface_container_lowest,
            ColorRole::SurfaceContainerLow => self.surface_container_low,
            ColorRole::SurfaceContainer => self.surface_container,
            ColorRole::SurfaceContainerHigh => self.surface_container_high,
            ColorRole::SurfaceContainerHighest => self.surface_container_highest,
            ColorRole::SurfaceTint => self.surface_tint,
            ColorRole::InverseSurface => self.inverse_surface,
            ColorRole::InverseOnSurface => self.inverse_on_surface,
            ColorRole::Background => self.background,
            ColorRole::OnBackground => self.on_background,
            ColorRole::Outline => self.outline,
            ColorRole::OutlineVariant => self.outline_variant,
            ColorRole::Shadow => self.shadow,
            ColorRole::Scrim => self.scrim,
        }
    }

    /// Every role with its color, in [`ColorRole::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (ColorRole, Color)> + '_ {
        ColorRole::ALL.iter().map(|&role| (role, self.get(role)))
    }
}

impl fmt::Debug for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|(role, color)| (role.name(), color)))
            .finish()
    }
}
