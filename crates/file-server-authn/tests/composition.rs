use authn::prelude::{AuthOutcome, Authenticate, Either, ExtractCredentials, Extracted};
use authn_axum::prelude::{Authn, AuthnRejection};
use authn_identity::prelude::{Identity, Machine, MachineId, User, UserId};
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
use std::{
    collections::HashMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
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

struct OidcAuthConfig {
    _client: OidcConfig,
    sessions: authn_oidc::prelude::Sessions,
    store: OidcUsers,
    cookies: authn_oidc_axum::prelude::CookieSettings,
}
impl OidcAuthConfig {
    fn new(client: OidcConfig, store: OidcUsers) -> Self {
        Self {
            _client: client,
            store,
            sessions: Default::default(),
            cookies: Default::default(),
        }
    }
}
struct TokenAuthConfig {
    store: Store,
}
struct TestConfig {
    oidc: Option<OidcAuthConfig>,
    token: Option<TokenAuthConfig>,
}
type OidcAuth = authn::prelude::Authenticator<
    authn_oidc_axum::prelude::SessionCookieExtractor,
    authn_oidc::prelude::OidcVerifier,
>;
type TokenAuth = authn::prelude::Authenticator<
    authn_token_axum::prelude::BearerTokenExtractor,
    authn_token::prelude::TokenVerifier<Store>,
>;
type TestAuth = (Option<OidcAuth>, Option<TokenAuth>);

fn authenticator(config: &TestConfig) -> TestAuth {
    use authn::prelude::Authenticator;
    (
        config.oidc.as_ref().map(|c| {
            Authenticator::new(
                c.cookies.extractor(),
                authn_oidc::prelude::OidcVerifier::new(c.sessions.clone(), c.store.clone()),
            )
        }),
        config.token.as_ref().map(|c| {
            Authenticator::new(
                authn_token_axum::prelude::BearerTokenExtractor,
                authn_token::prelude::TokenVerifier::new(c.store.clone()),
            )
        }),
    )
}

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
    let client = OidcConfig::new(
        "https://issuer.example",
        "client",
        "https://files.example/auth/callback",
    )
    .unwrap();
    let mut users = OidcUsers::default();
    users.insert(&client.issuer, "user-123", UserId::new("admin").unwrap());
    TestConfig {
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
async fn oidc_and_bearer_authenticate_into_the_common_identity() {
    let config = config();
    let auth = authenticator(&config);
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
async fn rejected_oidc_credentials_stop_before_bearer_fallback() {
    let config = config();
    let auth = authenticator(&config);
    let id = issue(&config);
    let cookie = format!("__Host-oidc-session={}", id.expose_secret());
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
    assert_eq!(
        authenticator(&config)
            .authenticate(&request(None, None))
            .await
            .unwrap(),
        AuthOutcome::NotApplicable
    );
    config.oidc = None;
    assert!(matches!(
        authenticator(&config)
            .authenticate(&request(
                Some("Bearer valid-token"),
                Some("__Host-oidc-session=stale")
            ))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(Identity::Machine(_))
    ));
    config.token = None;
    assert_eq!(
        authenticator(&config)
            .authenticate(&request(Some("Bearer valid-token"), None))
            .await
            .unwrap(),
        AuthOutcome::NotApplicable
    );
}

#[tokio::test]
async fn malformed_and_duplicate_credentials_are_rejected_without_store_lookup() {
    let config = config();
    let auth = authenticator(&config);
    for authorization in ["Bearer", "Bearer bad token", "Bearer\tvalid-token"] {
        assert_eq!(
            auth.authenticate(&request(Some(authorization), None))
                .await
                .unwrap(),
            AuthOutcome::Rejected
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
async fn token_store_failure_is_preserved_and_becomes_an_http_rejection() {
    let mut config = config();
    config.token.as_mut().unwrap().store.unavailable = true;
    let auth = authenticator(&config);
    let error = auth
        .authenticate(&request(Some("Bearer valid-token"), None))
        .await
        .unwrap_err();
    let Either::Right(Either::Right(error)) = error else {
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
        (
            Some("Bearer valid-token"),
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
fn token_debug_output_redacts_secrets_and_invalid_syntax_is_rejected() {
    assert!(!format!("{:?}", Token::new("valid-token").unwrap()).contains("valid-token"));
    assert!(Token::new("bad token").is_err());
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
async fn user_tokens_map_to_the_shared_identity() {
    let mut config = config();
    config.token.as_mut().unwrap().store.identities = Arc::new(HashMap::from([(
        Token::new("valid-token").unwrap().digest(),
        user("admin"),
    )]));
    assert_eq!(
        authenticator(&config)
            .authenticate(&request(Some("Bearer valid-token"), None))
            .await
            .unwrap(),
        AuthOutcome::Authenticated(user("admin"))
    );
}

#[tokio::test]
async fn unmapped_oidc_users_do_not_fall_back_to_bearer() {
    let mut config = config();
    let session = issue(&config);
    config.oidc.as_mut().unwrap().store = OidcUsers::default();
    let cookie = format!("__Host-oidc-session={}", session.expose_secret());
    assert_eq!(
        authenticator(&config)
            .authenticate(&request(Some("Bearer valid-token"), Some(&cookie)))
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
