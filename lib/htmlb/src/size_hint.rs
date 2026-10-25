/// Compile-time baseline for the HTML written by a value's type.
pub trait CompileTimeHtmlSizeHint {
    /// Bytes guaranteed by the value's type alone.
    const SIZE_HINT: usize;
}

/// Runtime addition to [`CompileTimeHtmlSizeHint::SIZE_HINT`] for value-dependent output.
pub trait RunTimeHtmlSizeHint {
    /// Additional estimated bytes beyond [`CompileTimeHtmlSizeHint::SIZE_HINT`].
    fn size_hint(&self) -> usize;
}
