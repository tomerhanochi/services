//! The corner-radius scale (`md.sys.shape.corner.*`).

/// How round a container's corners are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Shape {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    /// Pill or circle: half the shorter side.
    Full,
}

impl Shape {
    pub const ALL: &[Shape] = &[
        Shape::None,
        Shape::ExtraSmall,
        Shape::Small,
        Shape::Medium,
        Shape::Large,
        Shape::ExtraLarge,
        Shape::Full,
    ];

    /// The token's name in MD3, e.g. `full`.
    pub const fn name(self) -> &'static str {
        match self {
            Shape::None => "none",
            Shape::ExtraSmall => "extra-small",
            Shape::Small => "small",
            Shape::Medium => "medium",
            Shape::Large => "large",
            Shape::ExtraLarge => "extra-large",
            Shape::Full => "full",
        }
    }
}

impl Shape {
    /// The radius in CSS pixels (dp). [`Shape::Full`] is large enough to always clamp to a pill.
    pub const fn radius_px(self) -> u16 {
        match self {
            Self::None => 0,
            Self::ExtraSmall => 4,
            Self::Small => 8,
            Self::Medium => 12,
            Self::Large => 16,
            Self::ExtraLarge => 28,
            Self::Full => 9999,
        }
    }
}

/// Which corners a [`Shape`] rounds, e.g. a bottom sheet rounds only the top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Corners {
    All,
    Top,
    Bottom,
    Start,
    End,
}

impl Corners {
    pub const ALL: &[Corners] = &[
        Corners::All,
        Corners::Top,
        Corners::Bottom,
        Corners::Start,
        Corners::End,
    ];

    /// The token's name in MD3, e.g. `end`.
    pub const fn name(self) -> &'static str {
        match self {
            Corners::All => "all",
            Corners::Top => "top",
            Corners::Bottom => "bottom",
            Corners::Start => "start",
            Corners::End => "end",
        }
    }
}
