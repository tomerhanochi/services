use htmlb::Component;
use htmlb::prelude::*;

use crate::ColorRole;

const SURFACE: &str = "md-surface";

const fn role_class(role: ColorRole) -> &'static str {
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

/// A container filled with a color role; its text takes the matching content color.
pub fn surface(role: ColorRole) -> Surface<()> {
    Surface { role, content: () }
}

pub struct Surface<C> {
    role: ColorRole,
    content: C,
}

impl<C> Surface<C> {
    pub fn child<N: IntoHtml>(self, child: N) -> Surface<(C, N)> {
        Surface {
            role: self.role,
            content: (self.content, child),
        }
    }
}

impl<C: IntoHtml> Component for Surface<C> {
    fn render(self) -> impl IntoHtml {
        div()
            .class((SURFACE, role_class(self.role)))
            .child(self.content)
    }
    fn dynamic_len_hint(&self) -> usize {
        self.content.len_hint()
    }
}
