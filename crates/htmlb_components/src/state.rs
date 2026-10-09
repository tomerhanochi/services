//! Interaction states (`md.sys.state.*`): translucent layers of the content color drawn
//! over an interactive element.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StateLayer {
    Hover,
    Focus,
    Pressed,
    Dragged,
}

impl StateLayer {
    pub const ALL: &[StateLayer] = &[
        StateLayer::Hover,
        StateLayer::Focus,
        StateLayer::Pressed,
        StateLayer::Dragged,
    ];

    /// The token's name in MD3, e.g. `dragged`.
    pub const fn name(self) -> &'static str {
        match self {
            StateLayer::Hover => "hover",
            StateLayer::Focus => "focus",
            StateLayer::Pressed => "pressed",
            StateLayer::Dragged => "dragged",
        }
    }
}

impl StateLayer {
    pub const fn opacity(self) -> f32 {
        match self {
            Self::Hover => 0.08,
            Self::Focus | Self::Pressed => 0.12,
            Self::Dragged => 0.16,
        }
    }
}

/// Disabled content (text, icons) is the `on-*` color at this opacity.
pub const DISABLED_CONTENT_OPACITY: f32 = 0.38;
/// Disabled containers are `on-surface` at this opacity.
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
