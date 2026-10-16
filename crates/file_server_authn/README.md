# File server authentication

The composition authenticates requests in this order:

```rust
type StaticAuthenticator = Authenticator<BasicAuthExtractor, StaticVerifier<Identity>>;
type OidcAuthenticator = Authenticator<SessionCookieExtractor, OidcVerifier<Identity>>;
type TokenAuthenticator<S> = Authenticator<BearerTokenExtractor, TokenVerifier<S>>;
type AppAuthenticator<S> = (
    StaticAuthenticator,
    Option<OidcAuthenticator>,
    Option<TokenAuthenticator<S>>,
);
```

`S` is your token store. Every mechanism returns the same `Identity` enum, which
distinguishes the break-glass account, an OIDC identity, and an API token identity.
The configured Basic account produces `Identity::BreakGlass`.
Authorization of OIDC users and API token permissions belongs to the file server.

## Configuration and Axum

Import each authentication crate's public API with `use crate_name::prelude::*`.
Constructors parse untrusted configuration: `Username::new`, `Password::new`,
`Token::new`, and `OidcConfig::new` return `Result`. OIDC configuration stores
HTTPS issuer/callback URL types, a nonempty client ID, and positive lifetimes.
Use `PositiveDuration::new` to change a lifetime, `ClientSecret::new` for the
optional client secret, and `Scope::new` for additional scopes. Set a redirect
with `SuccessPath::new`, which permits local absolute paths.

```rust,ignore
let config = AuthConfig {
    r#static: StaticAuthConfig {
        username: Username::new(admin_username)?,
        password: Password::new(admin_password)?,
    },
    oidc: Some(OidcAuthConfig::new(oidc_client_config)),
    token: Some(TokenAuthConfig { store: your_database_store }),
};

// Discovery happens here. The session verifier and login routes share sessions.
let setup = configure(&config).await?;
let app = Router::new()
    .route("/", get(handler))
    .nest("/auth", setup.routes())
    .with_state(setup.authenticator);

async fn handler(Authn(identity): Authn<AppAuthenticator<YourStore>>) -> String {
    format!("{identity:?}")
}
```

`authenticator(&config)` also builds just the request authenticator synchronously,
without discovery. If used with a separately initialized `OidcClient`, pass the
same `config.oidc.sessions` to that client. Creating an independent session store
for the login routes would make their cookies unusable by the request verifier.
`None` disables OIDC or API tokens. Basic remains available independently of the
provider and token database.

## API token store

Implement `authn_token::prelude::TokenStore`. It receives a `TokenDigest` (SHA-256), not the
raw token, and returns `Result<Option<Identity>, YourError>`. Return `None` for
unknown, expired, disabled, or revoked tokens; return `Err` when the store cannot
perform the lookup. Store only digests of cryptographically random API tokens.
`Token::generate()` creates a random token; persist its `digest()` and return the
secret to its owner once. `Token::new(secret)?.digest()` supports provisioning
existing tokens.

The store may use a database, remote service, or any other persistence mechanism.
`TokenVerifier` is generic over the store and preserves its concrete error type.
`Arc<Store>` is supported. Stores must be cheaply cloneable or shared because
Axum obtains an authenticator from state for each request.

The example server contains a small in-memory store to demonstrate the port;
replace it with your persistent implementation.

## OIDC

OIDC login uses the [openidconnect library](https://docs.rs/openidconnect/4.0.1/openidconnect/)
for discovery, authorization-code exchange with PKCE, and signed ID token verification.
The callback checks browser-bound state, nonce, issuer, audience, expiry, and the
access-token hash when present. A login transaction is consumed once. Missing
signing keys trigger one provider refresh and verification retry.

With the routes nested under `/auth`, configure the provider's redirect URI as
`https://your-host/auth/callback`. `/auth/login` starts login, `/auth/callback`
issues the session cookie, and POST `/auth/logout` revokes the session and clears
the cookies. The success redirect defaults to `/` and must be a local path.

The cookie holds a random opaque session ID, never an ID/access token or identity.
Both the session and login cookies use `Secure`, `HttpOnly`, `SameSite=Lax`, and
`Path=/`, with no `Domain`. The default session cookie is `__Host-oidc-session`.
Change the cookie name with `CookieSettings::new(...)`.

Session expiry is capped by both `session_ttl` and the ID token's expiry. Sessions
are process-local, shared by clones, bounded to 10,000 active sessions, and removed
when expired. Pending logins expire and are bounded to 1,024 transactions. A restart
invalidates both. Multiple replicas need a shared session/login backend; the
current `Sessions` implementation is intended for a single process.

Issuer, provider endpoints, and callback must use HTTPS. A reverse proxy can
terminate HTTPS for the Axum application. Application routes that mutate state
using cookie authentication still need their own CSRF protection.

## Precedence and failures

| Request credentials | Result |
| --- | --- |
| Valid Basic account, with or without an OIDC cookie | Break-glass identity |
| Invalid/malformed Basic, even with a valid OIDC cookie | Rejected; 401 |
| Valid OIDC cookie and Bearer token | OIDC identity |
| Invalid/expired OIDC cookie and valid Bearer token | Rejected; 401 |
| Bearer token with no OIDC cookie | Token store lookup |
| Bearer token while OIDC is disabled | Token store lookup, ignoring OIDC cookies |
| No applicable credentials | 401 |
| Token store unavailable | 500; typed source retained |

Each extractor distinguishes its transport: Basic, Bearer, or the configured
session cookie. Extractors return `Extracted::Absent`, `Extracted::Malformed`, or
`Extracted::Present(parsed_credentials)`. Malformed input stops authentication
before verification. Duplicate matching Authorization
headers/cookies are rejected. Header scheme matching is case insensitive.

`Either` preserves which extraction, verification, or chain branch failed, and
implements `Error::source`. `AuthnRejection` retains the error when captured as an
extractor result. Default HTTP responses contain no internal error details.

The static verifier compares fixed-length credential digests in constant time.
This avoids comparisons that return early on a matching prefix. It does not turn
the configured plaintext password into a password database hash: configuration
must keep the password secret. Credential types redact secrets in `Debug` output.
Basic decoding follows the username/first-colon/password format from
[RFC 7617](https://www.rfc-editor.org/rfc/rfc7617.html).

## Run the example

Set `ADMIN_USERNAME` and `ADMIN_PASSWORD`, then run:

```sh
cargo run -p file_server_authn --example server
```

The server listens on `127.0.0.1:3000`. Set `API_TOKEN` to enable its example token
store. Set `OIDC_ISSUER`, `OIDC_CLIENT_ID`, and `OIDC_REDIRECT_URI` to enable OIDC;
`OIDC_CLIENT_SECRET` is optional for a public client. Serve it behind HTTPS for
OIDC cookies. The example identifies callers; it does not serve files.
