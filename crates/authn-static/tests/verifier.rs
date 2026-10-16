use authn::prelude::VerifyCredentials;
use authn_static::prelude::*;
use std::io;

fn credentials(value: &str) -> BasicCredentials {
    BasicCredentials::parse(value).unwrap()
}

#[derive(Clone)]
struct CustomStore;
impl StaticStore for CustomStore {
    type Identity = u64;
    type Error = io::Error;
    async fn lookup(&self, username: &Username) -> io::Result<Option<StaticAccount<u64>>> {
        match username.as_str() {
            "alice" => Ok(Some(StaticAccount::new(
                PasswordHash::new(&encoded_hash("secret")).unwrap(),
                42,
            ))),
            "unavailable" => Err(io::Error::other("store unavailable")),
            _ => Ok(None),
        }
    }
}
#[tokio::test]
async fn static_verification_supports_other_stores_and_preserves_errors() {
    let verifier = StaticVerifier::new(CustomStore);
    assert_eq!(
        verifier.verify(&credentials("alice:secret")).await.unwrap(),
        Some(42)
    );
    assert_eq!(
        verifier.verify(&credentials("alice:wrong")).await.unwrap(),
        None
    );
    assert_eq!(
        verifier
            .verify(&credentials("missing:secret"))
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        verifier
            .verify(&credentials("unavailable:secret"))
            .await
            .unwrap_err()
            .to_string(),
        "store unavailable"
    );
}

fn encoded_hash(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    // Deterministic test salt; real credentials must use a fresh random salt per hash.
    let salt = SaltString::encode_b64(b"test-salt-16bytes").unwrap();
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
