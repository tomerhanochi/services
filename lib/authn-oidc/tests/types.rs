use authn_oidc::prelude::*;
use std::time::Duration;

#[test]
fn configuration_parsing_rejects_invalid_fields() {
    assert!(
        OidcConfig::new(
            "https://issuer.example",
            "client",
            "https://app.example/callback"
        )
        .is_ok()
    );
    for (issuer, client, redirect) in [
        (
            "http://issuer.example",
            "client",
            "https://app.example/callback",
        ),
        ("https://issuer.example", "", "https://app.example/callback"),
        (
            "https://issuer.example",
            "client",
            "http://app.example/callback",
        ),
        ("invalid", "client", "https://app.example/callback"),
    ] {
        assert!(OidcConfig::new(issuer, client, redirect).is_err());
    }
    assert!(PositiveDuration::new(Duration::ZERO).is_err());
    assert!(PositiveDuration::new(Duration::MAX).is_err());
    assert_eq!(
        PositiveDuration::new(Duration::from_secs(1)).unwrap().get(),
        Duration::from_secs(1)
    );
}

#[test]
fn session_ids_require_a_canonical_256_bit_secret() {
    let valid = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert_eq!(
        SessionId::from_cookie(valid).unwrap().expose_secret(),
        valid
    );
    for value in [
        "",
        "stale",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
    ] {
        assert!(SessionId::from_cookie(value).is_err(), "{value}");
    }
    assert!(!format!("{:?}", SessionId::from_cookie(valid).unwrap()).contains(valid));
}
