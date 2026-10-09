use htmlb::Component;
use htmlb::prelude::*;

use crate::{ColorRole, TypeScale};

const fn type_class(scale: TypeScale) -> &'static str {
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

const fn color_class(role: ColorRole) -> &'static str {
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

/// Text in a [`TypeScale`] role: `<span>`, or `<p>` with [`paragraph`](Text::paragraph).
pub fn text<C: IntoHtml>(scale: TypeScale, content: C) -> Text<C> {
    Text {
        scale,
        content,
        color: None,
        paragraph: false,
    }
}

pub struct Text<C> {
    scale: TypeScale,
    content: C,
    color: Option<ColorRole>,
    paragraph: bool,
}

impl<C> Text<C> {
    /// Text color, e.g. [`ColorRole::OnSurfaceVariant`] for secondary text.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self
    }

    /// A block of its own (`<p>`) instead of inline text.
    pub fn paragraph(mut self) -> Self {
        self.paragraph = true;
        self
    }
}

impl<C: IntoHtml> Component for Text<C> {
    fn render(self) -> impl IntoHtml {
        let class = (
            type_class(self.scale),
            self.color.map(color_class),
        );
        match self.paragraph {
            true => Either::Left(p().class(class).child(self.content)),
            false => Either::Right(span().class(class).child(self.content)),
        }
    }
    fn dynamic_len_hint(&self) -> usize {
        self.content.len_hint()
    }
}
