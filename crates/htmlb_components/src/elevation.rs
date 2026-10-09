//! Elevation levels (`md.sys.elevation.level*`).

/// How far a surface sits above the one below it. Shown by a shadow and, for
/// surface-container colors, by tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Elevation {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl Elevation {
    pub const ALL: &[Elevation] = &[
        Elevation::Level0,
        Elevation::Level1,
        Elevation::Level2,
        Elevation::Level3,
        Elevation::Level4,
        Elevation::Level5,
    ];

    /// The token's name in MD3, e.g. `level5`.
    pub const fn name(self) -> &'static str {
        match self {
            Elevation::Level0 => "level0",
            Elevation::Level1 => "level1",
            Elevation::Level2 => "level2",
            Elevation::Level3 => "level3",
            Elevation::Level4 => "level4",
            Elevation::Level5 => "level5",
        }
    }
}

impl Elevation {
    /// Height in dp.
    pub const fn dp(self) -> u8 {
        match self {
            Self::Level0 => 0,
            Self::Level1 => 1,
            Self::Level2 => 3,
            Self::Level3 => 6,
            Self::Level4 => 8,
            Self::Level5 => 12,
        }
    }

    /// MD3's two-layer shadow as a CSS `box-shadow`: a tight key shadow (30%) and a wider
    /// ambient one (15%).
    pub const fn shadow(self) -> &'static str {
        match self {
            Self::Level0 => "none",
            Self::Level1 => "0 1px 2px rgb(0 0 0 / 0.3), 0 1px 3px 1px rgb(0 0 0 / 0.15)",
            Self::Level2 => "0 1px 2px rgb(0 0 0 / 0.3), 0 2px 6px 2px rgb(0 0 0 / 0.15)",
            Self::Level3 => "0 1px 3px rgb(0 0 0 / 0.3), 0 4px 8px 3px rgb(0 0 0 / 0.15)",
            Self::Level4 => "0 2px 3px rgb(0 0 0 / 0.3), 0 6px 10px 4px rgb(0 0 0 / 0.15)",
            Self::Level5 => "0 4px 4px rgb(0 0 0 / 0.3), 0 8px 12px 6px rgb(0 0 0 / 0.15)",
        }
    }
}
