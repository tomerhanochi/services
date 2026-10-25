# File-server authentication

`configure(&AuthConfig).await` constructs runtime stores, OIDC clients, sessions,
and the authenticator from serializable deployment inputs. Configuration contains
no pools, stores, session maps, or generic runtime objects.

```toml
# Required OIDC provider. Register /auth/callback with the provider.
[oidc]
issuer = "https://issuer.example"
client_id = "file-server"
redirect_uri = "https://files.example/auth/callback"
# client_secret may be supplied by an environment-variable override.
session_ttl_secs = 3600
login_ttl_secs = 600

[oidc.users]
provider-subject = "alice"

# Optional API-token database. Store only SHA-256 token digests.
[token]
database_url = "postgres://file-server:password@localhost/file-server"
max_connections = 5
```

OIDC is required; API-token authentication can be added with the optional
`[token]` section. Requests try OIDC session cookies, then bearer tokens.
Missing credentials permit fallback; present rejected credentials stop the chain. Both methods return
`authn_identity::prelude::Identity::{User, Machine}`.

OIDC users map verified `(issuer, subject)` pairs to local user IDs. Emails are
not account identifiers. Sessions and pending logins are bounded and process-local;
a restart invalidates them. Use a single replica until a shared backend is added.
Provider, issuer, and callback endpoints require HTTPS. Cookies remain Secure,
HttpOnly, and SameSite=Lax.

The `authn-token-postgres` adapter creates `authn_api_tokens` if needed. It contains `digest BYTEA` (32 bytes),
`identity JSONB`, optional `expires_at TIMESTAMPTZ`, and `revoked BOOLEAN`.
The identity JSON has the shared enum's serde representation, for example
`{"Machine":{"id":"backup"}}`. Unknown, expired, and revoked tokens reject;
database failures remain operational errors. Raw tokens are not persisted. Use
`Token::generate` for API tokens; keep its secret only on the
client and provision its `digest()` in PostgreSQL.
The adapter's configuration and store types are available from
`authn_token_postgres::prelude`.

`AuthSetup` exposes the `OidcHttp` state and the authenticator. The
application owns the OIDC endpoint paths and registers
`authn_oidc_axum::prelude::{login, callback, logout}` at those paths, passing the
`OidcHttp` value as route state. This keeps route ownership in the application
router. The `AppAuthenticator` alias is concrete. `authn` itself stays generic,
preserving `Authn<A>(A::Identity)` as a single-field tuple struct.
