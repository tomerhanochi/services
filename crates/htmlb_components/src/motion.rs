//! Easing curves and durations (`md.sys.motion.*`).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Easing {
    /// For most transitions.
    Standard,
    /// Leaving the screen.
    StandardAccelerate,
    /// Entering the screen.
    StandardDecelerate,
    /// For prominent transitions.
    Emphasized,
    EmphasizedAccelerate,
    EmphasizedDecelerate,
    Linear,
}

impl Easing {
    pub const ALL: &[Easing] = &[
        Easing::Standard,
        Easing::StandardAccelerate,
        Easing::StandardDecelerate,
        Easing::Emphasized,
        Easing::EmphasizedAccelerate,
        Easing::EmphasizedDecelerate,
        Easing::Linear,
    ];

    /// The token's name in MD3, e.g. `linear`.
    pub const fn name(self) -> &'static str {
        match self {
            Easing::Standard => "standard",
            Easing::StandardAccelerate => "standard-accelerate",
            Easing::StandardDecelerate => "standard-decelerate",
            Easing::Emphasized => "emphasized",
            Easing::EmphasizedAccelerate => "emphasized-accelerate",
            Easing::EmphasizedDecelerate => "emphasized-decelerate",
            Easing::Linear => "linear",
        }
    }
}

impl Easing {
    /// Control points `(x1, y1, x2, y2)` of a cubic Bézier.
    pub const fn cubic_bezier(self) -> [f32; 4] {
        match self {
            Self::Standard | Self::Emphasized => [0.2, 0.0, 0.0, 1.0],
            Self::StandardAccelerate => [0.3, 0.0, 1.0, 1.0],
            Self::StandardDecelerate => [0.0, 0.0, 0.0, 1.0],
            Self::EmphasizedAccelerate => [0.3, 0.0, 0.8, 0.15],
            Self::EmphasizedDecelerate => [0.05, 0.7, 0.1, 1.0],
            Self::Linear => [0.0, 0.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Duration {
    Short1,
    Short2,
    Short3,
    Short4,
    Medium1,
    Medium2,
    Medium3,
    Medium4,
    Long1,
    Long2,
    Long3,
    Long4,
    ExtraLong1,
    ExtraLong2,
    ExtraLong3,
    ExtraLong4,
}

impl Duration {
    pub const ALL: &[Duration] = &[
        Duration::Short1,
        Duration::Short2,
        Duration::Short3,
        Duration::Short4,
        Duration::Medium1,
        Duration::Medium2,
        Duration::Medium3,
        Duration::Medium4,
        Duration::Long1,
        Duration::Long2,
        Duration::Long3,
        Duration::Long4,
        Duration::ExtraLong1,
        Duration::ExtraLong2,
        Duration::ExtraLong3,
        Duration::ExtraLong4,
    ];

    /// The token's name in MD3, e.g. `extra-long4`.
    pub const fn name(self) -> &'static str {
        match self {
            Duration::Short1 => "short1",
            Duration::Short2 => "short2",
            Duration::Short3 => "short3",
            Duration::Short4 => "short4",
            Duration::Medium1 => "medium1",
            Duration::Medium2 => "medium2",
            Duration::Medium3 => "medium3",
            Duration::Medium4 => "medium4",
            Duration::Long1 => "long1",
            Duration::Long2 => "long2",
            Duration::Long3 => "long3",
            Duration::Long4 => "long4",
            Duration::ExtraLong1 => "extra-long1",
            Duration::ExtraLong2 => "extra-long2",
            Duration::ExtraLong3 => "extra-long3",
            Duration::ExtraLong4 => "extra-long4",
        }
    }
}

impl Duration {
    pub const fn millis(self) -> u16 {
        match self {
            Self::Short1 => 50,
            Self::Short2 => 100,
            Self::Short3 => 150,
            Self::Short4 => 200,
            Self::Medium1 => 250,
            Self::Medium2 => 300,
            Self::Medium3 => 350,
            Self::Medium4 => 400,
            Self::Long1 => 450,
            Self::Long2 => 500,
            Self::Long3 => 550,
            Self::Long4 => 600,
            Self::ExtraLong1 => 700,
            Self::ExtraLong2 => 800,
            Self::ExtraLong3 => 900,
            Self::ExtraLong4 => 1000,
        }
    }
}
