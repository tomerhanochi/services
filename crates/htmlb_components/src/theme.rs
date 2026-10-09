//! Theme values: the schemes and typefaces chosen by an application.

use crate::{Scheme, Typefaces};

/// Everything that varies between apps: color schemes and typefaces. The remaining MD3
/// tokens are fixed by the component library and exposed as CSS custom properties.
#[derive(Debug, Clone)]
pub struct Theme {
    pub light: Scheme,
    pub dark: Scheme,
    pub typefaces: Typefaces,
}

impl Theme {
    pub const fn new(light: Scheme, dark: Scheme, typefaces: Typefaces) -> Self {
        Self {
            light,
            dark,
            typefaces,
        }
    }
}
