//! The class names the components render and [`CSS`](crate::CSS) styles. Tests check that
//! every one has a rule.

use crate::{ButtonStyle, ColorRole, TypeScale};

pub const ICON: &str = "md-icon";

pub const BUTTON: &str = "md-button";
pub const BUTTON_LABEL: &str = "md-button__label";
pub const ICON_BUTTON: &str = "md-icon-button";

pub const TOP_APP_BAR: &str = "md-top-app-bar";
pub const TOP_APP_BAR_NAVIGATION: &str = "md-top-app-bar__navigation";
pub const TOP_APP_BAR_TITLES: &str = "md-top-app-bar__titles";
pub const TOP_APP_BAR_HEADLINE: &str = "md-top-app-bar__headline";
pub const TOP_APP_BAR_SUBTITLE: &str = "md-top-app-bar__subtitle";
pub const TOP_APP_BAR_ACTIONS: &str = "md-top-app-bar__actions";
pub const BOTTOM_APP_BAR: &str = "md-bottom-app-bar";

pub const LIST: &str = "md-list";
pub const LIST_ITEM: &str = "md-list-item";
pub const LIST_ITEM_MAIN: &str = "md-list-item__main";
pub const LIST_ITEM_LEADING: &str = "md-list-item__leading";
pub const LIST_ITEM_TEXT: &str = "md-list-item__text";
pub const LIST_ITEM_HEADLINE: &str = "md-list-item__headline";
pub const LIST_ITEM_SUPPORTING: &str = "md-list-item__supporting";
pub const LIST_ITEM_TRAILING_TEXT: &str = "md-list-item__trailing-text";
pub const LIST_ITEM_TRAILING: &str = "md-list-item__trailing";

pub const SHEET: &str = "md-sheet";
pub const SHEET_HANDLE: &str = "md-sheet__handle";
pub const SHEET_HEADLINE: &str = "md-sheet__headline";
pub const MENU_ITEM: &str = "md-menu-item";

pub const TEXT_FIELD: &str = "md-text-field";
pub const TEXT_FIELD_ERROR: &str = "md-text-field--error";
pub const TEXT_FIELD_INPUT: &str = "md-text-field__input";
pub const TEXT_FIELD_LABEL: &str = "md-text-field__label";
pub const TEXT_FIELD_SUPPORTING: &str = "md-text-field__supporting";

pub const SCAFFOLD: &str = "md-scaffold";
pub const SCAFFOLD_BODY: &str = "md-scaffold__body";
pub const PANE: &str = "md-pane";
pub const SNACKBAR: &str = "md-snackbar";
pub const DIVIDER: &str = "md-divider";

pub const fn type_scale(scale: TypeScale) -> &'static str {
    match scale {
        TypeScale::DisplayLarge => "md-type-display-large",
        TypeScale::DisplayMedium => "md-type-display-medium",
        TypeScale::DisplaySmall => "md-type-display-small",
        TypeScale::HeadlineLarge => "md-type-headline-large",
        TypeScale::HeadlineMedium => "md-type-headline-medium",
        TypeScale::HeadlineSmall => "md-type-headline-small",
        TypeScale::TitleLarge => "md-type-title-large",
        TypeScale::TitleMedium => "md-type-title-medium",
        TypeScale::TitleSmall => "md-type-title-small",
        TypeScale::BodyLarge => "md-type-body-large",
        TypeScale::BodyMedium => "md-type-body-medium",
        TypeScale::BodySmall => "md-type-body-small",
        TypeScale::LabelLarge => "md-type-label-large",
        TypeScale::LabelMedium => "md-type-label-medium",
        TypeScale::LabelSmall => "md-type-label-small",
    }
}

/// Text color only.
pub const fn text_color(role: ColorRole) -> &'static str {
    match role {
        ColorRole::Primary => "md-color-primary",
        ColorRole::OnPrimary => "md-color-on-primary",
        ColorRole::PrimaryContainer => "md-color-primary-container",
        ColorRole::OnPrimaryContainer => "md-color-on-primary-container",
        ColorRole::PrimaryFixed => "md-color-primary-fixed",
        ColorRole::PrimaryFixedDim => "md-color-primary-fixed-dim",
        ColorRole::OnPrimaryFixed => "md-color-on-primary-fixed",
        ColorRole::OnPrimaryFixedVariant => "md-color-on-primary-fixed-variant",
        ColorRole::InversePrimary => "md-color-inverse-primary",
        ColorRole::Secondary => "md-color-secondary",
        ColorRole::OnSecondary => "md-color-on-secondary",
        ColorRole::SecondaryContainer => "md-color-secondary-container",
        ColorRole::OnSecondaryContainer => "md-color-on-secondary-container",
        ColorRole::SecondaryFixed => "md-color-secondary-fixed",
        ColorRole::SecondaryFixedDim => "md-color-secondary-fixed-dim",
        ColorRole::OnSecondaryFixed => "md-color-on-secondary-fixed",
        ColorRole::OnSecondaryFixedVariant => "md-color-on-secondary-fixed-variant",
        ColorRole::Tertiary => "md-color-tertiary",
        ColorRole::OnTertiary => "md-color-on-tertiary",
        ColorRole::TertiaryContainer => "md-color-tertiary-container",
        ColorRole::OnTertiaryContainer => "md-color-on-tertiary-container",
        ColorRole::TertiaryFixed => "md-color-tertiary-fixed",
        ColorRole::TertiaryFixedDim => "md-color-tertiary-fixed-dim",
        ColorRole::OnTertiaryFixed => "md-color-on-tertiary-fixed",
        ColorRole::OnTertiaryFixedVariant => "md-color-on-tertiary-fixed-variant",
        ColorRole::Error => "md-color-error",
        ColorRole::OnError => "md-color-on-error",
        ColorRole::ErrorContainer => "md-color-error-container",
        ColorRole::OnErrorContainer => "md-color-on-error-container",
        ColorRole::Surface => "md-color-surface",
        ColorRole::OnSurface => "md-color-on-surface",
        ColorRole::SurfaceVariant => "md-color-surface-variant",
        ColorRole::OnSurfaceVariant => "md-color-on-surface-variant",
        ColorRole::SurfaceDim => "md-color-surface-dim",
        ColorRole::SurfaceBright => "md-color-surface-bright",
        ColorRole::SurfaceContainerLowest => "md-color-surface-container-lowest",
        ColorRole::SurfaceContainerLow => "md-color-surface-container-low",
        ColorRole::SurfaceContainer => "md-color-surface-container",
        ColorRole::SurfaceContainerHigh => "md-color-surface-container-high",
        ColorRole::SurfaceContainerHighest => "md-color-surface-container-highest",
        ColorRole::SurfaceTint => "md-color-surface-tint",
        ColorRole::InverseSurface => "md-color-inverse-surface",
        ColorRole::InverseOnSurface => "md-color-inverse-on-surface",
        ColorRole::Background => "md-color-background",
        ColorRole::OnBackground => "md-color-on-background",
        ColorRole::Outline => "md-color-outline",
        ColorRole::OutlineVariant => "md-color-outline-variant",
        ColorRole::Shadow => "md-color-shadow",
        ColorRole::Scrim => "md-color-scrim",
    }
}

/// Background color, plus the matching content color ([`ColorRole::content`]).
pub const fn surface(role: ColorRole) -> &'static str {
    match role {
        ColorRole::Primary => "md-surface-primary",
        ColorRole::OnPrimary => "md-surface-on-primary",
        ColorRole::PrimaryContainer => "md-surface-primary-container",
        ColorRole::OnPrimaryContainer => "md-surface-on-primary-container",
        ColorRole::PrimaryFixed => "md-surface-primary-fixed",
        ColorRole::PrimaryFixedDim => "md-surface-primary-fixed-dim",
        ColorRole::OnPrimaryFixed => "md-surface-on-primary-fixed",
        ColorRole::OnPrimaryFixedVariant => "md-surface-on-primary-fixed-variant",
        ColorRole::InversePrimary => "md-surface-inverse-primary",
        ColorRole::Secondary => "md-surface-secondary",
        ColorRole::OnSecondary => "md-surface-on-secondary",
        ColorRole::SecondaryContainer => "md-surface-secondary-container",
        ColorRole::OnSecondaryContainer => "md-surface-on-secondary-container",
        ColorRole::SecondaryFixed => "md-surface-secondary-fixed",
        ColorRole::SecondaryFixedDim => "md-surface-secondary-fixed-dim",
        ColorRole::OnSecondaryFixed => "md-surface-on-secondary-fixed",
        ColorRole::OnSecondaryFixedVariant => "md-surface-on-secondary-fixed-variant",
        ColorRole::Tertiary => "md-surface-tertiary",
        ColorRole::OnTertiary => "md-surface-on-tertiary",
        ColorRole::TertiaryContainer => "md-surface-tertiary-container",
        ColorRole::OnTertiaryContainer => "md-surface-on-tertiary-container",
        ColorRole::TertiaryFixed => "md-surface-tertiary-fixed",
        ColorRole::TertiaryFixedDim => "md-surface-tertiary-fixed-dim",
        ColorRole::OnTertiaryFixed => "md-surface-on-tertiary-fixed",
        ColorRole::OnTertiaryFixedVariant => "md-surface-on-tertiary-fixed-variant",
        ColorRole::Error => "md-surface-error",
        ColorRole::OnError => "md-surface-on-error",
        ColorRole::ErrorContainer => "md-surface-error-container",
        ColorRole::OnErrorContainer => "md-surface-on-error-container",
        ColorRole::Surface => "md-surface-surface",
        ColorRole::OnSurface => "md-surface-on-surface",
        ColorRole::SurfaceVariant => "md-surface-surface-variant",
        ColorRole::OnSurfaceVariant => "md-surface-on-surface-variant",
        ColorRole::SurfaceDim => "md-surface-surface-dim",
        ColorRole::SurfaceBright => "md-surface-surface-bright",
        ColorRole::SurfaceContainerLowest => "md-surface-surface-container-lowest",
        ColorRole::SurfaceContainerLow => "md-surface-surface-container-low",
        ColorRole::SurfaceContainer => "md-surface-surface-container",
        ColorRole::SurfaceContainerHigh => "md-surface-surface-container-high",
        ColorRole::SurfaceContainerHighest => "md-surface-surface-container-highest",
        ColorRole::SurfaceTint => "md-surface-surface-tint",
        ColorRole::InverseSurface => "md-surface-inverse-surface",
        ColorRole::InverseOnSurface => "md-surface-inverse-on-surface",
        ColorRole::Background => "md-surface-background",
        ColorRole::OnBackground => "md-surface-on-background",
        ColorRole::Outline => "md-surface-outline",
        ColorRole::OutlineVariant => "md-surface-outline-variant",
        ColorRole::Shadow => "md-surface-shadow",
        ColorRole::Scrim => "md-surface-scrim",
    }
}

/// Added to [`BUTTON`] for each style.
pub const fn button(style: ButtonStyle) -> &'static str {
    match style {
        ButtonStyle::Filled => "md-button-filled",
        ButtonStyle::Tonal => "md-button-tonal",
        ButtonStyle::Elevated => "md-button-elevated",
        ButtonStyle::Outlined => "md-button-outlined",
        ButtonStyle::Text => "md-button-text",
    }
}
