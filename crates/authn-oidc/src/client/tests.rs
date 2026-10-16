use super::*;

use axum::{
    Json, Router,
    extract::{Form, State},
    routing::post,
};
use chrono::Utc;
use openidconnect::{
    AccessToken, Audience, AuthUrl, ClientId, ClientSecret, EmptyAdditionalClaims, IssuerUrl,
    JsonWebKeySetUrl, ResponseTypes, StandardClaims, SubjectIdentifier, TokenUrl,
    core::{
        CoreHmacKey, CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreResponseType,
        CoreSubjectIdentifierType,
    },
};
use serde_json::json;

const SECRET: &str = "test-client-secret-with-at-least-32-bytes";

#[derive(Clone, Copy)]
enum ResponseMode {
    Valid,
    WrongNonce,
    WrongAudience,
    WrongIssuer,
    Expired,
    WrongAccessToken,
    BadSignature,
}

#[derive(Clone)]
struct Provider {
    nonce: Arc<Mutex<String>>,
    pkce: Arc<Mutex<String>>,
    mode: ResponseMode,
}

async fn token_endpoint(
    State(provider): State<Provider>,
    Form(form): Form<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    assert_eq!(form.get("grant_type").unwrap(), "authorization_code");
    assert_eq!(form.get("code").unwrap(), "test-code");
    assert_eq!(
        form.get("code_verifier").unwrap(),
        &*provider.pkce.lock().unwrap()
    );
    let nonce = if matches!(provider.mode, ResponseMode::WrongNonce) {
        "wrong".into()
    } else {
        provider.nonce.lock().unwrap().clone()
    };
    let audience = if matches!(provider.mode, ResponseMode::WrongAudience) {
        "other-client"
    } else {
        "client"
    };
    let issuer = if matches!(provider.mode, ResponseMode::WrongIssuer) {
        "https://other.example"
    } else {
        "https://issuer.example"
    };
    let expires = if matches!(provider.mode, ResponseMode::Expired) {
        Utc::now() - chrono::Duration::seconds(60)
    } else {
        Utc::now() + chrono::Duration::hours(1)
    };
    let access_token = AccessToken::new("test-access-token".into());
    let secret = if matches!(provider.mode, ResponseMode::BadSignature) {
        "different-secret-with-at-least-32-bytes"
    } else {
        SECRET
    };
    let token = CoreIdToken::new(
        CoreIdTokenClaims::new(
            IssuerUrl::new(issuer.into()).unwrap(),
            vec![Audience::new(audience.into())],
            expires,
            Utc::now(),
            StandardClaims::new(SubjectIdentifier::new("subject-123".into())),
            EmptyAdditionalClaims {},
        )
        .set_nonce(Some(Nonce::new(nonce))),
        &CoreHmacKey::new(secret),
        CoreJwsSigningAlgorithm::HmacSha256,
        Some(&access_token),
        None,
    )
    .unwrap();
    Json(
        json!({ "access_token": if matches!(provider.mode, ResponseMode::WrongAccessToken) { "substituted" } else { access_token.secret() },
        "token_type": "Bearer", "id_token": token.to_string() }),
    )
}

async fn fixture(mode: ResponseMode) -> (OidcClient, Provider, tokio::task::JoinHandle<()>) {
    let provider = Provider {
        nonce: Arc::new(Mutex::new(String::new())),
        pkce: Arc::new(Mutex::new(String::new())),
        mode,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let token_url = format!("http://{}/token", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/token", post(token_endpoint))
        .with_state(provider.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let metadata = CoreProviderMetadata::new(
        IssuerUrl::new("https://issuer.example".into()).unwrap(),
        AuthUrl::new("https://issuer.example/authorize".into()).unwrap(),
        JsonWebKeySetUrl::new("https://issuer.example/jwks".into()).unwrap(),
        vec![ResponseTypes::new(vec![CoreResponseType::Code])],
        vec![CoreSubjectIdentifierType::Public],
        vec![CoreJwsSigningAlgorithm::HmacSha256],
        Default::default(),
    )
    .set_token_endpoint(Some(TokenUrl::new(token_url).unwrap()));
    // Test-only construction allows a loopback token endpoint; production requires HTTPS.
    let mut config = OidcConfig::new(
        "https://issuer.example",
        "client",
        "https://files.example/auth/callback",
    )
    .unwrap();
    config.client_secret = Some(ClientSecret::new(SECRET.into()));
    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new("client".into()),
        Some(ClientSecret::new(SECRET.into())),
    )
    .set_redirect_uri(config.redirect_uri.inner().clone());
    (
        OidcClient {
            inner: Arc::new(Inner {
                client: Mutex::new(client),
                config,
                http: reqwest::Client::new(),
                sessions: Sessions::default(),
                pending: Mutex::new(HashMap::new()),
            }),
        },
        provider,
        task,
    )
}

fn login(client: &OidcClient, provider: &Provider) -> LoginRequest {
    let request = client.begin_login().unwrap();
    let pending = client.inner.pending.lock().unwrap();
    let login = pending.get(request.state.secret()).unwrap();
    *provider.nonce.lock().unwrap() = login.nonce.secret().clone();
    *provider.pkce.lock().unwrap() = login.pkce.secret().clone();
    let url = openidconnect::url::Url::parse(&request.url).unwrap();
    let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query.get("code_challenge_method").unwrap(), "S256");
    assert!(
        query
            .get("scope")
            .unwrap()
            .split_whitespace()
            .any(|scope| scope == "openid")
    );
    request
}

#[tokio::test]
async fn signed_callback_issues_a_session_and_cannot_be_replayed() {
    let (client, provider, server) = fixture(ResponseMode::Valid).await;
    let request = login(&client, &provider);
    assert!(matches!(
        client
            .complete_login("test-code", request.state.secret(), "wrong-browser")
            .await,
        Err(OidcError::InvalidCallback)
    ));
    let id = client
        .complete_login("test-code", request.state.secret(), request.state.secret())
        .await
        .unwrap();
    let identity = client.sessions().lookup(&id).unwrap().unwrap();
    assert_eq!(identity.subject, "subject-123");
    assert_eq!(identity.issuer, "https://issuer.example");
    assert!(matches!(
        client
            .complete_login("test-code", request.state.secret(), request.state.secret())
            .await,
        Err(OidcError::InvalidCallback)
    ));
    client.sessions().revoke(&id).unwrap();
    assert!(client.sessions().lookup(&id).unwrap().is_none());
    server.abort();
}

#[tokio::test]
async fn rejects_invalid_nonce_audience_issuer_expiry_access_token_and_signature() {
    for mode in [
        ResponseMode::WrongNonce,
        ResponseMode::WrongAudience,
        ResponseMode::WrongIssuer,
        ResponseMode::Expired,
        ResponseMode::WrongAccessToken,
        ResponseMode::BadSignature,
    ] {
        let (client, provider, server) = fixture(mode).await;
        let request = login(&client, &provider);
        assert!(
            client
                .complete_login("test-code", request.state.secret(), request.state.secret())
                .await
                .is_err()
        );
        server.abort();
    }
}

#[tokio::test]
async fn expired_login_and_insecure_configuration_are_rejected() {
    let (client, provider, server) = fixture(ResponseMode::Valid).await;
    let request = login(&client, &provider);
    client
        .inner
        .pending
        .lock()
        .unwrap()
        .get_mut(request.state.secret())
        .unwrap()
        .expires_at = SystemTime::UNIX_EPOCH;
    assert!(matches!(
        client
            .complete_login("test-code", request.state.secret(), request.state.secret())
            .await,
        Err(OidcError::InvalidCallback)
    ));
    assert!(matches!(
        OidcConfig::new(
            "http://issuer.example",
            "client",
            "https://files.example/auth/callback"
        ),
        Err(OidcError::Configuration(_))
    ));
    server.abort();
}
