//! Number formatting via `core::fmt::NumBuffer` (stable since Rust 1.98).

use core::fmt::NumBuffer;

/// Exact width of `n` in decimal, for size estimates.
#[inline]
pub(crate) fn digits(n: u64) -> usize {
    n.checked_ilog10().map_or(1, |d| d as usize + 1)
}

#[inline]
pub(crate) fn digits_signed(n: i64) -> usize {
    (n < 0) as usize + digits(n.unsigned_abs())
}

#[inline]
pub(crate) fn write_u64(buf: &mut String, n: u64) {
    buf.push_str(n.format_into(&mut NumBuffer::new()))
}

#[inline]
pub(crate) fn write_i64(buf: &mut String, n: i64) {
    buf.push_str(n.format_into(&mut NumBuffer::new()))
}

#[inline]
pub(crate) fn write_display(buf: &mut String, v: impl std::fmt::Display) {
    use std::fmt::Write;
    let _ = write!(buf, "{v}");
}
