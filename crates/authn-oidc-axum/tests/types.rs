use authn::prelude::*;
use authn_oidc_axum::prelude::*;
use axum::http::{Request, header::COOKIE};

#[test]
fn redirects_are_parsed_as_local_header_safe_paths() {
    for value in [
        "",
        "https://other.example",
        "//other.example",
        "/\\other.example",
        "/path\r\nheader: value",
    ] {
        assert!(SuccessPath::new(value).is_err(), "{value:?}");
    }
    assert_eq!(
        SuccessPath::new("/files?sort=name").unwrap().as_str(),
        "/files?sort=name"
    );
}

#[test]
fn both_cookie_names_must_fit_the_limit() {
    assert!(CookieName::new("a".repeat(128)).is_ok());
    assert!(CookieName::new("a".repeat(129)).is_err());
    assert!(CookieSettings::new("a".repeat(122)).is_ok());
    assert!(CookieSettings::new("a".repeat(123)).is_err());
}

#[test]
fn malformed_cookies_are_distinct_from_absent_cookies() {
    let extractor = SessionCookieExtractor::default();
    let parts = Request::builder().body(()).unwrap().into_parts().0;
    assert!(matches!(
        extractor.extract(&parts).unwrap(),
        Extracted::Absent
    ));
    for value in [
        "__Host-oidc-session=",
        "__Host-oidc-session=stale",
        "__Host-oidc-session",
        "__Host-oidc-session=a; __Host-oidc-session=b",
    ] {
        let parts = Request::builder()
            .header(COOKIE, value)
            .body(())
            .unwrap()
            .into_parts()
            .0;
        assert!(matches!(
            extractor.extract(&parts).unwrap(),
            Extracted::Malformed
        ));
    }
}
