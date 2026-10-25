# File-server authentication

`configure(&AuthConfig).await` constructs runtime stores, OIDC clients, sessions,
and the authenticator from serializable deployment inputs. Configuration contains
no pools, stores, session maps, or generic runtime objects.

```toml
[static]
directory = "/run/secrets/file-users"

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
try Basic, OIDC session cookies, then bearer tokens. Missing credentials permit
fallback; present malformed or rejected credentials stop the chain. All methods
return `authn_identity::prelude::Identity::{User, Machine}`.

Static account filenames are local user IDs; files contain Argon2id v19 PHC hashes
with at least 19 MiB memory, two iterations, 16-byte salts, and 32-byte outputs.
Generate independently salted hashes before provisioning the directory as a
Kubernetes Secret. A single LF or CRLF is accepted at the end of each hash file.
The store uses Tokio filesystem operations, follows projected-volume symlinks,
and reads files afresh on each attempt. The directory and its symlinks must be
controlled by the deployment; the path-based adapter is not a capability sandbox.

OIDC users map verified `(issuer, subject)` pairs to local user IDs. Emails are
not account identifiers. Sessions and pending logins are bounded and process-local;
a restart invalidates them. Use a single replica until a shared backend is added.
Provider, issuer, and callback endpoints require HTTPS. Cookies remain Secure,
HttpOnly, and SameSite=Lax.

Setup creates `authn_api_tokens` if needed. It contains `digest BYTEA` (32 bytes),
`identity JSONB`, optional `expires_at TIMESTAMPTZ`, and `revoked BOOLEAN`.
The identity JSON has the shared enum's serde representation, for example
`{"Machine":{"id":"backup"}}`. Unknown, expired, and revoked tokens reject;
database failures remain operational errors. Neither raw tokens nor passwords
are persisted. Use `Token::generate` for API tokens; keep its secret only on the
client and provision its `digest()` in PostgreSQL.

Mount `setup.routes()` under `/auth` and use `setup.authenticator` as Axum state.
The `AppAuthenticator` alias is concrete. `authn` itself stays generic, preserving
`Authn<A>(A::Identity)` as a single-field tuple struct.
