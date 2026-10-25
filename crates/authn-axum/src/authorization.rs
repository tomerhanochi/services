use authn::prelude::Extracted;
use axum::http::request::Parts;
/// Extracts one Authorization credential for `scheme` (case insensitive).
/// Matching malformed or ambiguous headers are explicit parsing failures.
pub fn authorization<'a>(parts: &'a Parts, scheme: &str) -> Extracted<&'a [u8]> {
    let values = parts.headers.get_all(axum::http::header::AUTHORIZATION);
    let count = values.iter().count();
    for value in values {
        let bytes = value.as_bytes();
        let end = bytes
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(bytes.len());
        if !bytes[..end].eq_ignore_ascii_case(scheme.as_bytes()) {
            continue;
        }
        if count != 1 || bytes.get(end) != Some(&b' ') {
            return Extracted::Malformed;
        }
        let credential = &bytes[end..];
        let start = credential
            .iter()
            .position(|byte| *byte != b' ')
            .unwrap_or(credential.len());
        return Extracted::Present(&credential[start..]);
    }
    Extracted::Absent
}
