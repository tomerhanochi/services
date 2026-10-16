use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use authn_static::prelude::*;

fn encoded_hash() -> String {
    let salt = SaltString::encode_b64(b"test-salt-16bytes").unwrap();
    Argon2::default()
        .hash_password(b"correct-password", &salt)
        .unwrap()
        .to_string()
}

#[test]
fn only_complete_supported_secure_password_hashes_can_be_constructed() {
    let valid = encoded_hash();
    let hash = PasswordHash::new(&valid).unwrap();
    assert!(!format!("{hash:?}").contains(&valid));
    assert!(!format!("{:?}", StaticAccount::new(hash, "user")).contains(&valid));
    let weak_memory = valid.replace("m=19456", "m=8");
    let weak_iterations = valid.replace("t=2", "t=1");
    let old_version = valid.replace("v=19", "v=16");
    let wrong_algorithm = valid.replace("argon2id", "argon2i");
    let extra_parameter = valid.replace("p=1", "p=1,unknown=1");
    let missing_hash = valid.rsplit_once('$').unwrap().0;
    for value in [
        "",
        "plaintext-password",
        "5e884898da280471",
        &weak_memory,
        &weak_iterations,
        &old_version,
        &wrong_algorithm,
        &extra_parameter,
        missing_hash,
    ] {
        assert!(PasswordHash::new(value).is_err(), "{value}");
    }
    assert!(PasswordHash::new(&(valid + "\n")).is_err());
}

#[test]
fn short_salts_and_short_hash_outputs_are_rejected() {
    let salt = SaltString::encode_b64(b"shortsalt").unwrap();
    let short_salt = Argon2::default()
        .hash_password(b"password", &salt)
        .unwrap()
        .to_string();
    assert!(PasswordHash::new(&short_salt).is_err());
    let params = argon2::Params::new(19456, 2, 1, Some(16)).unwrap();
    let argon = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = SaltString::encode_b64(b"test-salt-16bytes").unwrap();
    let short_output = argon.hash_password(b"password", &salt).unwrap().to_string();
    assert!(PasswordHash::new(&short_output).is_err());
}
