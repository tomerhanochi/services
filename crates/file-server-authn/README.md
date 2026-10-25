# File-server authentication

`configure(&AuthConfig).await` constructs runtime stores, OIDC clients, sessions,
and the authenticator from serializable deployment inputs. Configuration contains
no pools, stores, session maps, or generic runtime objects.

```toml
# Optional OIDC provider. Register /auth/callback with the provider.
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

Each section is optional, but at least one mechanism must be enabled. Requests
try OIDC session cookies, then bearer tokens. Missing credentials permit fallback;
present rejected credentials stop the chain. Both methods return
`authn_identity::prelude::Identity::{User, Machine}`.

OIDC users map verified `(issuer, subject)` pairs to local user IDs. Emails are
not account identifiers. Sessions and pending logins are bounded and process-local;
a restart invalidates them. Use a single replica until a shared backend is added.
Provider, issuer, and callback endpoints require HTTPS. Cookies remain Secure,
HttpOnly, and SameSite=Lax.

Setup creates `authn_api_tokens` if needed. It contains `digest BYTEA` (32 bytes),
`identity JSONB`, optional `expires_at TIMESTAMPTZ`, and `revoked BOOLEAN`.
The identity JSON has the shared enum's serde representation, for example
`{"Machine":{"id":"backup"}}`. Unknown, expired, and revoked tokens reject;
database failures remain operational errors. Raw tokens are not persisted. Use
`Token::generate` for API tokens; keep its secret only on the
client and provision its `digest()` in PostgreSQL.

Mount `setup.routes()` under `/auth` and use `setup.authenticator` as Axum state.
The `AppAuthenticator` alias is concrete. `authn` itself stays generic, preserving
`Authn<A>(A::Identity)` as a single-field tuple struct.
