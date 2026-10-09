//! The type scale (`md.sys.typescale.*`) and typefaces (`md.ref.typeface.*`).

/// A role in the type scale: five roles, each in three sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TypeScale {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

impl TypeScale {
    pub const ALL: &[TypeScale] = &[
        TypeScale::DisplayLarge,
        TypeScale::DisplayMedium,
        TypeScale::DisplaySmall,
        TypeScale::HeadlineLarge,
        TypeScale::HeadlineMedium,
        TypeScale::HeadlineSmall,
        TypeScale::TitleLarge,
        TypeScale::TitleMedium,
        TypeScale::TitleSmall,
        TypeScale::BodyLarge,
        TypeScale::BodyMedium,
        TypeScale::BodySmall,
        TypeScale::LabelLarge,
        TypeScale::LabelMedium,
        TypeScale::LabelSmall,
    ];

    /// The token's name in MD3, e.g. `label-small`.
    pub const fn name(self) -> &'static str {
        match self {
            TypeScale::DisplayLarge => "display-large",
            TypeScale::DisplayMedium => "display-medium",
            TypeScale::DisplaySmall => "display-small",
            TypeScale::HeadlineLarge => "headline-large",
            TypeScale::HeadlineMedium => "headline-medium",
            TypeScale::HeadlineSmall => "headline-small",
            TypeScale::TitleLarge => "title-large",
            TypeScale::TitleMedium => "title-medium",
            TypeScale::TitleSmall => "title-small",
            TypeScale::BodyLarge => "body-large",
            TypeScale::BodyMedium => "body-medium",
            TypeScale::BodySmall => "body-small",
            TypeScale::LabelLarge => "label-large",
            TypeScale::LabelMedium => "label-medium",
            TypeScale::LabelSmall => "label-small",
        }
    }
}

/// Which of the theme's [`Typefaces`] a style uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Typeface {
    /// For large text: display, headline, title-large.
    Brand,
    /// For everything else.
    Plain,
}

/// The two font families a theme provides. Each is a CSS-style family list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Typefaces {
    pub brand: &'static str,
    pub plain: &'static str,
}

impl Typefaces {
    pub const fn get(&self, typeface: Typeface) -> &'static str {
        match typeface {
            Typeface::Brand => self.brand,
            Typeface::Plain => self.plain,
        }
    }
}

/// How text in one [`TypeScale`] role looks. Lengths are in `rem` (16px at default zoom),
/// so text follows the user's font-size preference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    pub typeface: Typeface,
    pub weight: u16,
    pub size_rem: f32,
    pub line_height_rem: f32,
    pub tracking_rem: f32,
}

const fn style(
    typeface: Typeface,
    weight: u16,
    size_rem: f32,
    line_height_rem: f32,
    tracking_rem: f32,
) -> TypeStyle {
    TypeStyle {
        typeface,
        weight,
        size_rem,
        line_height_rem,
        tracking_rem,
    }
}

const REGULAR: u16 = 400;
const MEDIUM: u16 = 500;

impl TypeScale {
    /// The baseline style for this role.
    pub const fn style(self) -> TypeStyle {
        use Typeface::{Brand, Plain};
        match self {
            Self::DisplayLarge => style(Brand, REGULAR, 3.5625, 4.0, -0.015625),
            Self::DisplayMedium => style(Brand, REGULAR, 2.8125, 3.25, 0.0),
            Self::DisplaySmall => style(Brand, REGULAR, 2.25, 2.75, 0.0),
            Self::HeadlineLarge => style(Brand, REGULAR, 2.0, 2.5, 0.0),
            Self::HeadlineMedium => style(Brand, REGULAR, 1.75, 2.25, 0.0),
            Self::HeadlineSmall => style(Brand, REGULAR, 1.5, 2.0, 0.0),
            Self::TitleLarge => style(Brand, REGULAR, 1.375, 1.75, 0.0),
            Self::TitleMedium => style(Plain, MEDIUM, 1.0, 1.5, 0.009375),
            Self::TitleSmall => style(Plain, MEDIUM, 0.875, 1.25, 0.00625),
            Self::BodyLarge => style(Plain, REGULAR, 1.0, 1.5, 0.03125),
            Self::BodyMedium => style(Plain, REGULAR, 0.875, 1.25, 0.015625),
            Self::BodySmall => style(Plain, REGULAR, 0.75, 1.0, 0.025),
            Self::LabelLarge => style(Plain, MEDIUM, 0.875, 1.25, 0.00625),
            Self::LabelMedium => style(Plain, MEDIUM, 0.75, 1.0, 0.03125),
            Self::LabelSmall => style(Plain, MEDIUM, 0.6875, 1.0, 0.03125),
        }
    }
}
