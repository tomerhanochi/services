use authn_identity::prelude::{Identity, Machine, MachineId, User, UserId};
use authn_static::prelude::{Password, StaticAccount, StaticStore, Username};
use authn_static_directory::prelude::DirectoryStore;
use std::{
    collections::HashMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
};

use authn::prelude::{AuthOutcome, Authenticate, Either, ExtractCredentials, Extracted};
use authn_axum::prelude::{Authn, AuthnRejection};
use authn_oidc::prelude::{OidcConfig, OidcIdentity, OidcUsers, SessionId};
use authn_token::prelude::{Token, TokenDigest, TokenStore};
use axum::{
    Router,
    body::Body,
    extract::FromRequestParts,
    http::{
        Request, StatusCode,
        header::{AUTHORIZATION, COOKIE},
        request::Parts,
    },
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use file_server_authn::prelude::{
    AppAuthenticator, AuthConfig, OidcAuthConfig, StaticAuthConfig, TokenAuthConfig, authenticator,
};
use tower::ServiceExt;

#[derive(Debug, Clone, Default)]
struct Store {
    identities: Arc<HashMap<TokenDigest, Identity>>,
    calls: Arc<AtomicUsize>,
    unavailable: bool,
}

impl TokenStore for Store {
    type Identity = Identity;
    type Error = io::Error;

    async fn lookup(&self, digest: &TokenDigest) -> Result<Option<Identity>, io::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        if self.unavailable {
            return Err(io::Error::other("database unavailable"));
        }
        Ok(self.identities.get(digest).cloned())
    }
}

#[derive(Debug, Clone)]
struct MountedUsers {
    store: DirectoryStore,
    _directory: Arc<tempfile::TempDir>,
}
impl StaticStore for MountedUsers {
    type Identity = Identity;
    type Error = io::Error;
    async fn lookup(&self, username: &Username) -> io::Result<Option<StaticAccount<Identity>>> {
        self.store.lookup(username).await
    }
}
type TestConfig = AuthConfig<Store, MountedUsers>;
type TestAuth = AppAuthenticator<Store, MountedUsers>;

fn user(id: &str) -> Identity {
    User {
        id: UserId::new(id).unwrap(),
    }
    .into()
}
fn machine(id: &str) -> Identity {
    Machine {
        id: MachineId::new(id).unwrap(),
    }
    .into()
}

fn config() -> TestConfig {
    let directory = Arc::new(tempfile::tempdir().unwrap());
    std::fs::write(
        directory.path().join("admin"),
        encoded_hash("secret:with:colons"),
    )
    .unwrap();
    let client = OidcConfig::new(
        "https://issuer.example",
        "client",
        "https://files.example/auth/callback",
    )
    .unwrap();
    let mut users = OidcUsers::default();
    users.insert(&client.issuer, "user-123", UserId::new("admin").unwrap());
    AuthConfig {
        r#static: StaticAuthConfig {
            store: MountedUsers {
                store: DirectoryStore::new(directory.path()),
                _directory: directory,
            },
        },
        oidc: Some(OidcAuthConfig::new(client, users)),
        token: Some(TokenAuthConfig {
            store: Store {
                identities: Arc::new(HashMap::from([(
                    Token::new("valid-token").unwrap().digest(),
                    machine("service"),
                )])),
                ..Store::default()
            },
        }),
    }
}

fn request(authorization: Option<&str>, cookie: Option<&str>) -> Parts {
    let mut builder = Request::builder();
    if let Some(value) = authorization {
        builder = builder.header(AUTHORIZATION, value);
    }
    if let Some(value) = cookie {
        builder = builder.header(COOKIE, value);
    }
    builder.body(()).unwrap().into_parts().0
}

fn basic(value: &str) -> String {
    format!("Basic {}", STANDARD.encode(value))
}

fn issue(config: &TestConfig) -> SessionId {
    config
        .oidc
        .as_ref()
        .unwrap()
        .sessions
        .issue(
            OidcIdentity {
                issuer: "https://issuer.example".into(),
                subject: "user-123".into(),
                email: None,
                email_verified: false,
            },
            SystemTime::now() + Duration::from_secs(3600),
        )
        .unwrap()
}

#[tokio::test]
async fn each_mechanism_authenticates_into_the_common_identity() {
    let config = config();
    let auth = authenticator(&config);
    assert_eq!(
        auth.authenticate(&request(Some(&basic("admin:secret:with:colons")), None))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(user("admin"))
    );
    let id = issue(&config);
    let cookie = format!("theme=dark; __Host-oidc-session={}", id.expose_secret());
    assert_eq!(
        auth.authenticate(&request(None, Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(user("admin"))
    );
    assert_eq!(
        auth.authenticate(&request(Some("bEaReR   valid-token"), None))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(machine("service"))
    );
    assert_eq!(
        config
            .token
            .as_ref()
            .unwrap()
            .store
            .calls
            .load(Ordering::SeqCst),
        1
    );
}

#[tokio::test]
async fn precedence_and_rejection_stop_the_chain() {
    let config = config();
    let auth = authenticator(&config);
    let id = issue(&config);
    let cookie = format!("__Host-oidc-session={}", id.expose_secret());
    assert!(matches!(
        auth.authenticate(&request(
            Some(&basic("admin:secret:with:colons")),
            Some(&cookie)
        ))
        .await
        .unwrap(),
        AuthOutcome::Authenticated(Identity::User(_))
    ));
    assert_eq!(
        auth.authenticate(&request(Some(&basic("admin:wrong")), Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Rejected
    );
    assert!(matches!(
        auth.authenticate(&request(Some("Bearer valid-token"), Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(Identity::User(_))
    ));
    config.oidc.as_ref().unwrap().sessions.revoke(&id).unwrap();
    assert_eq!(
        auth.authenticate(&request(Some("Bearer valid-token"), Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Rejected
    );
    assert_eq!(
        config
            .token
            .as_ref()
            .unwrap()
            .store
            .calls
            .load(Ordering::SeqCst),
        0
    );
}

#[tokio::test]
async fn absent_and_disabled_mechanisms_are_not_applicable() {
    let mut config = config();
    let auth = authenticator(&config);
    assert_eq!(
        auth.authenticate(&request(None, None)).await.unwrap(),
        AuthOutcome::NotApplicable
    );
    config.oidc = None;
    let auth = authenticator(&config);
    assert!(matches!(
        auth.authenticate(&request(
            Some("Bearer valid-token"),
            Some("__Host-oidc-session=stale")
        ))
        .await
        .unwrap(),
        AuthOutcome::Authenticated(Identity::Machine(_))
    ));
    config.token = None;
    let auth = authenticator(&config);
    assert_eq!(
        auth.authenticate(&request(Some("Bearer valid-token"), None))
            .await
            .unwrap(),
        AuthOutcome::NotApplicable
    );
    assert!(matches!(
        auth.authenticate(&request(Some(&basic("admin:secret:with:colons")), None))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(Identity::User(_))
    ));
}

#[tokio::test]
async fn malformed_and_duplicate_credentials_are_rejected_without_store_lookup() {
    let config = config();
    let auth = authenticator(&config);
    for authorization in [
        "Basic",
        "Basic invalid!",
        "Bearer",
        "Bearer bad token",
        "Bearer\tvalid-token",
    ] {
        assert_eq!(
            auth.authenticate(&request(Some(authorization), None))
                .await
                .unwrap(),
            AuthOutcome::Rejected,
            "{authorization}"
        );
    }
    let mut parts = request(Some("Bearer valid-token"), None);
    parts
        .headers
        .append(AUTHORIZATION, "Bearer other-token".parse().unwrap());
    assert_eq!(
        auth.authenticate(&parts).await.unwrap(),
        AuthOutcome::Rejected
    );
    let id = issue(&config);
    let cookie = format!(
        "__Host-oidc-session={0}; __Host-oidc-session={0}",
        id.expose_secret()
    );
    assert_eq!(
        auth.authenticate(&request(None, Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Rejected
    );
    assert_eq!(
        config
            .token
            .as_ref()
            .unwrap()
            .store
            .calls
            .load(Ordering::SeqCst),
        0
    );
}

#[tokio::test]
async fn token_store_failure_is_preserved_in_the_nested_error() {
    let mut config = config();
    config.token.as_mut().unwrap().store.unavailable = true;
    let auth = authenticator(&config);
    let error = auth
        .authenticate(&request(Some("Bearer valid-token"), None))
        .await
        .unwrap_err();
    let Either::Right(Either::Right(Either::Right(error))) = error else {
        panic!("expected token verification error")
    };
    assert_eq!(error.to_string(), "database unavailable");
    let result = Authn::<TestAuth>::from_request_parts(
        &mut request(Some("Bearer valid-token"), None),
        &auth,
    )
    .await;
    assert!(matches!(result, Err(AuthnRejection::Authentication(_))));
}

#[tokio::test]
async fn composition_works_in_a_real_axum_router() {
    async fn handler(Authn(identity): Authn<TestAuth>) -> String {
        format!("{identity:?}")
    }
    let mut config = config();
    config.token.as_mut().unwrap().store.unavailable = true;
    let app = Router::new()
        .route("/", get(handler))
        .with_state(authenticator(&config));
    for (authorization, status) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some(basic("admin:secret:with:colons")), StatusCode::OK),
        (Some(basic("admin:wrong")), StatusCode::UNAUTHORIZED),
        (
            Some("Bearer valid-token".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        let mut request = Request::builder().uri("/");
        if let Some(value) = authorization {
            request = request.header(AUTHORIZATION, value);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}

#[test]
fn invalid_config_is_rejected_and_secrets_are_redacted() {
    let config = config();
    let debug = format!("{config:?}");
    assert!(!debug.contains("secret:with:colons"));
    assert!(!format!("{:?}", Token::new("valid-token").unwrap()).contains("valid-token"));
    assert!(Password::new("").is_err());
    assert!(Username::new("user:with:colon").is_err());
}

#[tokio::test]
async fn custom_cookie_names_and_multiple_cookie_headers_work() {
    let mut config = config();
    config.oidc.as_mut().unwrap().cookies =
        authn_oidc_axum::prelude::CookieSettings::new("files_session").unwrap();
    let id = issue(&config);
    let mut parts = request(None, Some("theme=dark"));
    parts.headers.append(
        COOKIE,
        format!("files_session={}", id.expose_secret())
            .parse()
            .unwrap(),
    );
    let extractor = config.oidc.as_ref().unwrap().cookies.extractor();
    assert!(matches!(
        extractor.extract(&parts).unwrap(),
        Extracted::Present(_)
    ));
    assert!(matches!(
        authenticator(&config).authenticate(&parts).await.unwrap(),
        AuthOutcome::Authenticated(Identity::User(_))
    ));
}

#[tokio::test]
async fn user_tokens_return_the_same_identity_as_basic_and_oidc() {
    let mut config = config();
    config.token.as_mut().unwrap().store.identities = Arc::new(HashMap::from([(
        Token::new("valid-token").unwrap().digest(),
        user("admin"),
    )]));
    let auth = authenticator(&config);
    assert_eq!(
        auth.authenticate(&request(Some("Bearer valid-token"), None))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(user("admin"))
    );
}

#[tokio::test]
async fn unmapped_oidc_users_are_rejected_without_falling_back_to_bearer() {
    let mut config = config();
    let session = issue(&config);
    config.oidc.as_mut().unwrap().store = OidcUsers::default();
    let auth = authenticator(&config);
    let cookie = format!("__Host-oidc-session={}", session.expose_secret());
    assert_eq!(
        auth.authenticate(&request(Some("Bearer valid-token"), Some(&cookie)))
            .await
            .unwrap(),
        AuthOutcome::Rejected
    );
    assert_eq!(
        config
            .token
            .as_ref()
            .unwrap()
            .store
            .calls
            .load(Ordering::SeqCst),
        0
    );
}

#[tokio::test]
async fn malformed_password_files_preserve_the_store_error_and_return_500() {
    let config = config();
    std::fs::write(
        config.r#static.store._directory.path().join("admin"),
        "invalid\n",
    )
    .unwrap();
    let auth = authenticator(&config);
    let parts = request(Some(&basic("admin:secret:with:colons")), None);
    let error = auth.authenticate(&parts).await.unwrap_err();
    let Either::Left(Either::Right(Either::Left(error))) = error else {
        panic!("expected static store error")
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    let mut parts = parts;
    let rejection = Authn::<TestAuth>::from_request_parts(&mut parts, &auth).await;
    assert!(matches!(rejection, Err(AuthnRejection::Authentication(_))));
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
