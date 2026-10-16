use authn_static::prelude::*;

#[test]
fn basic_parsing_preserves_password_colons_and_unicode() {
    let credentials = BasicCredentials::parse("admin:päss:word").unwrap();
    assert_eq!(credentials.username(), "admin");
    assert_eq!(credentials.password(), "päss:word");
    assert!(!format!("{credentials:?}").contains("päss"));
}

#[test]
fn static_access_fields_cannot_contain_invalid_values() {
    for username in ["", "user:name", "user\nname"] {
        assert!(Username::new(username).is_err());
    }
    for password in ["", "pass\rword"] {
        assert!(Password::new(password).is_err());
    }
    for value in [
        "missing-separator",
        ":password",
        "username:",
        "user:pass\nword",
    ] {
        assert!(BasicCredentials::parse(value).is_err());
    }
    assert!(!format!("{:?}", Password::new("secret").unwrap()).contains("secret"));
}
