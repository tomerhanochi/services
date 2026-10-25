use file_server_authn::prelude::*;

#[test]
fn oidc_and_token_settings_parse_without_runtime_resources() {
    let config: AuthConfig = toml::from_str(
        "[oidc]\nissuer='https://issuer.example'\nclient_id='files'\nredirect_uri='https://files.example/auth/callback'\n[oidc.users]\nsubject='alice'\n[token]\ndatabase_url='postgres://user:secret@localhost/db'",
    )
    .unwrap();
    assert!(config.oidc.is_some());
    assert!(config.token.is_some());
    assert!(!format!("{config:?}").contains("secret"));
}

#[tokio::test]
async fn empty_configuration_is_rejected() {
    assert!(matches!(
        configure(&AuthConfig::default()).await,
        Err(BuildError::NoMechanisms)
    ));
}

#[test]
fn secrets_are_redacted_and_store_fields_are_not_configuration() {
    let config: AuthConfig =
        toml::from_str("[token]\ndatabase_url='postgres://user:secret@localhost/db'").unwrap();
    assert!(!format!("{config:?}").contains("secret"));
    assert!(toml::from_str::<AuthConfig>("[static]\ndirectory='users'").is_err());
}

#[test]
fn zero_database_pool_sizes_are_rejected_during_parsing() {
    assert!(
        toml::from_str::<AuthConfig>(
            "[token]\ndatabase_url='postgres://localhost/db'\nmax_connections=0"
        )
        .is_err()
    );
}
