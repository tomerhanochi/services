# File server authentication

Requests try HTTP Basic, an OIDC session cookie, then an API bearer token. Absent
credentials allow the next mechanism; present but invalid credentials stop the
chain. Every mechanism returns `authn_identity::prelude::Identity`, containing
`User { id: UserId }` or `Machine { id: MachineId }`. The identity does not expose
the authentication method. Basic and OIDC can resolve to the same local user;
a token's store may return either a user or a machine.

Import public APIs through each crate's inline `prelude` module. `authn` remains
generic, so Axum still extracts `Authn<A>(A::Identity)`.

## Configuration

```rust,ignore
use authn_identity::prelude::*;
use authn_oidc::prelude::*;
use authn_static::prelude::*;
use authn_static_directory::prelude::*;
use file_server_authn::prelude::*;
use cap_std::{fs::Dir, ambient_authority};

let client = OidcConfig::new(
    "https://issuer.example",
    "client-id",
    "https://files.example/auth/callback",
)?;
let mut users = OidcUsers::default();
users.insert(&client.issuer, "provider-subject", UserId::new("alice")?);

let config = AuthConfig {
    r#static: StaticAuthConfig {
        store: DirectoryStore::new(Dir::open_ambient_dir(
            "/run/secrets/file-users", ambient_authority(),
        )?),
    },
    oidc: Some(OidcAuthConfig::new(client, users)),
    token: Some(TokenAuthConfig { store: your_token_store }),
};
let setup = configure(&config).await?;
let app = Router::new()
    .route("/", get(handler))
    .nest("/auth", setup.routes())
    .with_state(setup.authenticator);

async fn handler(Authn(identity): Authn<AppAuthenticator<YourTokenStore>>) -> String {
    format!("{identity:?}")
}
```

`AuthConfig<T, S = DirectoryStore, O = OidcUsers>` and
`AppAuthenticator<T, S = DirectoryStore, O = OidcUsers>` support custom token,
static-account, and OIDC identity stores. Stores must be cheaply cloneable or
shared through `Arc`, because Axum obtains an authenticator from application
state per request.

`authenticator(&config)` constructs just the request authenticator synchronously
and infallibly. `configure(&config).await` also discovers the OIDC provider and
builds login routes sharing the configured sessions. `None` disables OIDC or
bearer authentication. Basic uses the configured store independently of them.

## Mounted static accounts

`authn-static` provides `StaticStore`, `StaticAccount<I>`, and
`StaticVerifier<S>`. A store looks up a parsed `Username`, returning its identity
and a parsed `PasswordHash`, or `None` for unknown or disabled users. The account
constructor accepts only a secure hash:

```rust,ignore
let hash = PasswordHash::new(encoded_phc_hash)?;
let account = StaticAccount::new(hash, identity);
```

`PasswordHash` accepts Argon2id v19 PHC strings with at least 19 MiB memory,
two iterations, a 16-byte salt, and a 32-byte output. It rejects plaintext,
unsupported algorithms, malformed encodings, and hashes below these minimums.
Generate hashes with a fresh random salt per account before provisioning them.
Hash verification runs on a blocking worker. Wrong passwords reject
authentication; store, hash-verification, and worker failures preserve their
source through `Either` and `PasswordVerificationError`.

`authn-static-directory` supplies `DirectoryStore`, which accepts an already-open
`cap_std::fs::Dir`. Opening an ambient path is the application's responsibility;
the store retains only the directory capability. Each filename identifies a local
`UserId`, and its contents must be an Argon2id PHC hash:

```text
/run/secrets/file-users/
    alice       # contents: $argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>
    bob         # contents: another independently salted Argon2id PHC hash
```

The store opens and reads each file through `Dir` on a blocking worker for every
lookup. It follows Kubernetes' projected symlinks within the granted directory;
cap-std enforces confinement during filesystem operations. No canonicalization
or preflight filesystem metadata checks precede the open. Errors from opening
and reading are handled directly. The capability continues referring to the
same directory if its ambient path is renamed or replaced.

Usernames must be single filenames containing ASCII letters, digits, `-`, `_`,
or `.`, at most 255 bytes long; `.` and names starting with `..` are excluded.
Missing users reject authentication. Read failures and invalid hash files are
operational errors. Hash files must be UTF-8, at most 1 KiB long, and may have a
single trailing LF or CRLF line ending. No other whitespace is trimmed.

A Kubernetes Secret can supply multiple users:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: file-users
type: Opaque
stringData:
  alice: "$argon2id$v=19$m=19456,t=2,p=1$<alice-salt>$<alice-hash>"
  bob: "$argon2id$v=19$m=19456,t=2,p=1$<bob-salt>$<bob-hash>"
```

Replace the hash placeholders with complete, independently salted PHC strings.
Mount the whole Secret directory read-only:

```yaml
volumes:
  - name: file-users
    secret:
      secretName: file-users
containers:
  - name: file-server
    # ...
    env:
      - name: STATIC_AUTH_DIRECTORY
        value: /run/secrets/file-users
    volumeMounts:
      - name: file-users
        mountPath: /run/secrets/file-users
        readOnly: true
```

Kubernetes propagates Secret updates eventually. Individual `subPath` mounts do
not receive those updates; see the [Kubernetes Secret documentation](https://kubernetes.io/docs/concepts/configuration/secret/#using-secrets-as-files-from-a-pod).

## API token store

Implement `authn_token::prelude::TokenStore` with `type Identity = Identity` from
`authn-identity`. Lookups receive `TokenDigest`, not the raw token. Return only
active, unexpired, unrevoked principals; unknown tokens return `None`, and store
failures return `Err`. `Arc<Store>` is supported.

`Token::generate()` creates a random token. Persist its digest and expose the
secret to its owner once. `Token::new(secret)?.digest()` parses existing tokens.
The example's in-memory token store should be replaced with a persistent store.

## OIDC identity mapping

`OidcIdentity` contains verified provider claims and remains internal to OIDC
resolution. `OidcIdentityStore` maps those claims to the shared application
identity, allowing an asynchronous user database lookup. Unknown or disabled
users return `None`; lookup failures remain typed errors. `Arc<Store>` is
supported. The supplied `OidcUsers` stores explicit `(issuer, subject) -> UserId`
mappings; email does not identify an account.

OIDC login uses authorization-code exchange with PKCE. Token verification checks
nonce, issuer, audience, expiry, signature, and the access-token hash when
present. Browser-bound state is checked, and login transactions are single-use.
Missing signing keys trigger one provider refresh and retry.

When nested under `/auth`, `/auth/login` starts login, `/auth/callback` issues a
session, and POST `/auth/logout` revokes it. Both cookies use `Secure`,
`HttpOnly`, `SameSite=Lax`, and `Path=/`, with no `Domain`. Sessions are opaque,
process-local, shared by clones, and bounded to 10,000; pending logins are bounded
to 1,024. A restart invalidates both. Session expiry is capped by the configured
lifetime and ID token expiry. Multiple replicas need a shared session backend.

Issuer, provider endpoints, and callback must use HTTPS. A reverse proxy may
terminate HTTPS. Use `CookieSettings::new` for a custom cookie name,
`PositiveDuration::new` for lifetimes, and `SuccessPath::new` for a local success
redirect. Cookie-authenticated state-changing application routes need CSRF
protection.

## Failures

Invalid, malformed, duplicated, expired, or unmapped credentials produce a 401
and stop fallback. Operational failures produce a 500 with the original error
retained through `Either` and `AuthnRejection`. Default HTTP responses do not
expose internal error details. Credential types redact secrets in `Debug` output.

## Example

Set `STATIC_AUTH_DIRECTORY` to the mounted password-hash directory, then run:

```sh
cargo run -p file_server_authn --example server
```

The server listens on `127.0.0.1:3000`. Set `API_TOKEN` to enable its example
machine token. To enable OIDC, set `OIDC_ISSUER`, `OIDC_CLIENT_ID`,
`OIDC_REDIRECT_URI`, `OIDC_SUBJECT`, and `OIDC_USER_ID`. The latter two explicitly
map one provider subject to a local user; use an identity store for larger user
populations. `OIDC_CLIENT_SECRET` is optional for public clients.
