# File server

A working Axum binary for browsing, downloading, renaming, and moving files in
configured mounts. The restored legacy application uses the current `authn*`,
`htmlb`, and `htmlb_components` crates. PostgreSQL stores role assignments, mount
grants, and optional API-token digests through SQLx's PostgreSQL driver with
Rustls. There is no Turso or unit-of-work dependency.

## Run locally

Install Rust, Just, and Docker, then run from the repository root. The live smoke
test also uses Python 3:

```sh
just postgres-up
mkdir -p /tmp/file-server-local/users /tmp/file-server-local/documents
cp crates/file_server/file_server.example.toml /tmp/file-server-local/server.toml
printf '%s' 'choose-a-password' | just hash-password > /tmp/file-server-local/users/alice
just run /tmp/file-server-local/server.toml
```

Open <http://localhost:8080> and choose **Sign in**. The browser prompts for Basic
credentials: username `alice`, with the password you hashed. Put files in
`/tmp/file-server-local/documents`. Only independently salted Argon2id hashes go
in the users directory. HTTP Basic credentials are managed by the client; the
logout route clears OIDC sessions, not browser-cached Basic credentials.

The example Compose database binds localhost port 15432 and uses development
credentials. `just postgres-up 15434` overrides its published port. Run `just postgres-down` to stop it; its named volume retains data.

Build the standalone binary with `just build-release`; run
`file_server --help` for the config path rules. The binary handles SIGINT and
SIGTERM for graceful shutdown. `/healthz` reports liveness; startup connects to
PostgreSQL and applies embedded SQL migrations before serving.

## Configuration

See [file_server.example.toml](file_server.example.toml). All inputs are literal
configuration values. Startup constructs stores, connection pools, services,
OIDC clients, and sessions internally.

A positional config path takes precedence over `FILE_SERVER_CONFIG`, whose
default is `file_server.toml`. The TOML file is merged with `FILE_SERVER_*`
environment overrides. Double underscores separate nested fields:

```sh
FILE_SERVER_DATABASE__URL='postgres://user:password@db/files' \
FILE_SERVER_LISTEN='0.0.0.0:8080' \
FILE_SERVER_PUBLIC_URL='https://files.example' \
just run /path/to/server.toml
```

Mount paths and static-auth directory paths are relative to the TOML file's
parent directory. `public_url` must be an HTTP(S) origin. Database URLs and OIDC
client secrets are redacted from config Debug output. Unknown configuration
fields, duplicate mounts, and dangling role/mount references fail during parsing.

`[roles.NAME]` maps mounts to `viewer` or `editor`. `[users]` and `[machines]`
assign roles to local principal IDs. A viewer can list and download; an editor
can also rename and move. Unlisted principals receive no grants. User and machine
IDs occupy separate namespaces. On startup, config-owned role memberships and
grants are replaced atomically in PostgreSQL; token records are left intact.

`[auth.static]` selects an Argon2id hash directory. Optional `[auth.oidc]` contains
provider settings and explicit subject-to-local-user mappings. The verified
issuer and subject identify the mapping; email and provider roles are not used
as local account IDs. Optional `[auth.token]` specifies the PostgreSQL token-store
connection. See [file_server_authn](../file_server_authn/README.md) for the exact
settings and token table schema. A token may identify a user or a machine.

OIDC routes are `/auth/login`, `/auth/callback`, and POST `/auth/logout`.
Configure HTTPS at the external URL and provider; OIDC cookies are always Secure.
OIDC sessions and pending logins currently remain bounded process-local state,
so use one replica and expect sessions to end on restart.

## Filesystem behavior

File operations use Tokio. Paths are parsed into mount names and relative path
components. Resolved symlinks outside a mount are rejected; renames use an atomic
no-replace operation so existing destinations are preserved. Moves stay within
one mount. Atomic mutations currently require Unix filesystem support for
`RENAME_NOREPLACE` / `RENAME_EXCL`.

The path-based adapters assume mount directories and their symlinks are managed
by trusted code. Canonical path checks do not sandbox concurrent hostile
filesystem writers. `cap-std` is intentionally deferred until an async capability
adapter is selected.

The UI uses the current Material components and generated theme CSS. Downloads
are streamed, filenames are escaped, mutation routes check request origin, and
responses include security and private-cache headers.

## Test with Docker

```sh
just test-postgres
```

The recipe creates its own disposable PostgreSQL Compose project on localhost
port 15433, runs the entire workspace suite including database tests, then runs
the built binary through a live HTTP smoke test. It verifies TOML/environment
merging, authentication, listing, downloading, renaming, and graceful shutdown.
Its container and volume are removed on exit. Pass a port, such as `just test-postgres 15434`, to override the test port. Do not point the database integration tests at production: they replace
config-owned access records.

For an existing disposable PostgreSQL container:

```sh
TEST_DATABASE_URL='postgres://user:password@localhost:15432/file_server' \
just test-all
```

Ordinary `just test` runs without PostgreSQL; the three
database tests are explicitly ignored unless requested. No database is needed
to compile the binary: SQLx queries use runtime binding, and migrations are
embedded in the PostgreSQL adapter.

`just` lists all available commands. Use `just build`, `just check`, and `just fmt`
for development. The smoke test is a recipe in the same Justfile; no standalone
build or test scripts are required.
