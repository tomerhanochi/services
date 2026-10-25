//! A human-readable size, written without allocating.

use std::fmt;

use htmlb::prelude::{Component, IntoHtml, RunTimeHtmlSizeHint};

/// `512 B`, `1.5 KB`, `12 MB`, ... (powers of 1000, as file managers on phones show them).
#[derive(Clone, Copy)]
pub struct Size(pub u64);

const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
const POWERS: [u64; 6] = [
    1,
    1_000,
    1_000_000,
    1_000_000_000,
    1_000_000_000_000,
    1_000_000_000_000_000,
];

/// A size as shown, rounded half up: the formatting and the length estimate share it.
enum Shown {
    /// `1.5 KB`, in tenths: below 10 of the unit.
    Tenths(u64, &'static str),
    /// `12 MB`
    Whole(u64, &'static str),
}

impl Size {
    fn shown(self) -> Shown {
        let n = self.0;
        let unit = n
            .checked_ilog(1000)
            .map_or(0, |u| (u as usize).min(UNITS.len() - 1));
        if unit == 0 {
            return Shown::Whole(n, UNITS[unit]);
        }
        let power = POWERS[unit];
        // Below 10 of the unit, so `n * 10` can't overflow.
        if n < 10 * power {
            let tenths = (n * 10 + power / 2) / power;
            if tenths < 100 {
                return Shown::Tenths(tenths, UNITS[unit]);
            }
        }
        let whole = n / power + u64::from(n % power * 2 >= power);
        match whole {
            // `999.5 KB` rounds up to the next unit, not to `1000 KB`.
            1000 if unit < UNITS.len() - 1 => Shown::Tenths(10, UNITS[unit + 1]),
            _ => Shown::Whole(whole, UNITS[unit]),
        }
    }
}

fn digits(n: u64) -> usize {
    n.checked_ilog10().map_or(1, |d| d as usize + 1)
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.shown() {
            Shown::Tenths(t, unit) => write!(f, "{}.{} {unit}", t / 10, t % 10),
            Shown::Whole(n, unit) => write!(f, "{n} {unit}"),
        }
    }
}

impl Component for Size {
    fn render(self) -> impl IntoHtml {
        htmlb::prelude::display(self)
    }
}
impl RunTimeHtmlSizeHint for Size {
    fn size_hint(&self) -> usize {
        // Exact.
        match self.shown() {
            Shown::Tenths(_, unit) => "1.5 ".len() + unit.len(),
            Shown::Whole(n, unit) => digits(n) + 1 + unit.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        let s = |n| Size(n).to_string();
        assert_eq!(s(0), "0 B");
        assert_eq!(s(999), "999 B");
        assert_eq!(s(1500), "1.5 KB");
        assert_eq!(s(1250), "1.3 KB");
        assert_eq!(s(9_949), "9.9 KB");
        assert_eq!(s(9_960), "10 KB");
        assert_eq!(s(12_345_678), "12 MB");
        assert_eq!(s(999_499), "999 KB");
        assert_eq!(s(999_500), "1.0 MB");
        assert_eq!(s(u64::MAX), "18447 PB");
    }

    #[test]
    fn size_hints_are_exact() {
        let edges = [
            0, 9, 10, 999, 1000, 1250, 9_949, 9_950, 9_960, 99_499, 99_500,
        ];
        let more = [
            999_499,
            999_500,
            12_345_678,
            999_500_000_000_000_000,
            u64::MAX,
        ];
        for n in edges.into_iter().chain(more) {
            let size = Size(n);
            assert_eq!(
                <Size as CompileTimeHtmlSizeHint>::SIZE_HINT + size.size_hint(),
                size.to_html().len(),
                "{n}"
            );
        }
    }
}
