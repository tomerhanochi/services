//! HTML escaping, delegated to `askama_escape`: `& < > " '` become numeric entities
//! (`&#38;` ...). The same escaper serves text and (double-quoted) attribute values.

#[inline]
pub(crate) fn write(buf: &mut String, s: &str) {
    // Writing to a `String` can't fail.
    let _ = askama_escape::escape_html(buf, s);
}
