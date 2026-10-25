//! A human-readable date, written without allocating.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use htmlb::{Component, IntoHtml};

/// `2024-05-01` (UTC).
#[derive(Clone, Copy)]
pub struct Date(pub SystemTime);

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days = self
            .0
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() / 86_400) as i64;
        let (y, m, d) = civil_from_days(days);
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

impl Component for Date {
    fn render(self) -> impl IntoHtml {
        htmlb::display(self)
    }
    fn dynamic_len_hint(&self) -> usize {
        // `YYYY-MM-DD`; longer only after the year 9999.
        10
    }
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 to (year, month, day).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn dates() {
        let d = |secs| Date(UNIX_EPOCH + Duration::from_secs(secs)).to_string();
        assert_eq!(d(0), "1970-01-01");
        assert_eq!(d(951_782_400), "2000-02-29");
        assert_eq!(d(1_791_072_000), "2026-10-04");
        let date = Date(UNIX_EPOCH);
        assert_eq!(date.len_hint(), date.to_html().len());
    }
}
