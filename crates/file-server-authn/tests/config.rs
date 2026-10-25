use authn::prelude::{AuthOutcome, Authenticate};
use axum::http::Request;
use file_server_authn::prelude::*;

#[tokio::test]
async fn config_values_build_a_real_directory_authenticator() {
    let directory = tempfile::tempdir().unwrap();
    let config: AuthConfig = toml::from_str(&format!(
        "[static]\ndirectory = {:?}",
        directory.path().to_str().unwrap()
    ))
    .unwrap();
    let setup = configure(&config).await.unwrap();
    let parts = Request::new(()).into_parts().0;
    assert_eq!(
        setup.authenticator.authenticate(&parts).await.unwrap(),
        AuthOutcome::NotApplicable
    );
    assert!(config.oidc.is_none());
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
    assert!(toml::from_str::<AuthConfig>("[static]\nstore='not-config'").is_err());
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
