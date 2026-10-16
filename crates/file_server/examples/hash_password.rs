use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;
    let password = password
        .strip_suffix('\n')
        .map(|p| p.strip_suffix('\r').unwrap_or(p))
        .unwrap_or(&password);
    if password.is_empty() {
        return Err("password is empty".into());
    }
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    println!(
        "{}",
        Argon2::default().hash_password(password.as_bytes(), &salt)?
    );
    Ok(())
}
