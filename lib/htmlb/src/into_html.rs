use crate::size_hint::{CompileTimeHtmlSizeHint, RunTimeHtmlSizeHint};

/// Anything that can be rendered as HTML: elements, text, numbers, and containers of them.
#[diagnostic::on_unimplemented(message = "`{Self}` can't be rendered as HTML")]
pub trait IntoHtml: CompileTimeHtmlSizeHint + RunTimeHtmlSizeHint + Sized {
    /// Appends the HTML to `buf`. Does not reserve.
    fn write_html(self, buf: &mut String);

    /// Reserves [`SIZE_HINT`](CompileTimeHtmlSizeHint::SIZE_HINT) plus the runtime addition and
    /// 1/8 headroom for escape expansion, then renders. To reuse a buffer, call
    /// [`write_html`](Self::write_html).
    #[hotpath::measure]
    fn to_html(self) -> String {
        let hint = Self::SIZE_HINT + self.size_hint();
        let mut buf = String::with_capacity(hint + hint / 8);
        self.write_html(&mut buf);
        buf
    }
}
