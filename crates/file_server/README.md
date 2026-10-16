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

## Profile requests

Instrumentation follows the [profiling guide](https://hotpath.rs/blog/profiling-rust-guide)
and [Grafana guide](https://hotpath.rs/blog/rust-performance-grafana). Normal builds
leave all hotpath macros inactive. Enabling the executable's `hotpath` feature
activates the shared profiler dependency across the libraries through Cargo
feature unification. Individual instrumented libraries also expose `hotpath`
for use by other executables; the executable must create a profiling guard.

```sh
just profile /tmp/file-server-local/server.toml
# Or match the unoptimized `just run` build when investigating development latency:
just profile-debug /tmp/file-server-local/server.toml
```

Make requests in the browser, then press Ctrl-C for a report with p50/p95/p99
function timings, SQL, routes, I/O and async-future activity. The authenticated
request path includes credential extraction, account lookup, Argon2 verification,
access queries, filesystem operations, page construction, and HTML serialization.
OIDC discovery and login operations have function timings. SQLx tracing is
installed separately from the console log filter, so `RUST_LOG=info` still captures
SQL query timings and their function/route attribution.

The Prometheus exporter is at <http://127.0.0.1:6772/metrics>. Optional allocation
tracking is available with `just profile-alloc CONFIG`; use separate timing and
allocation runs when comparing latency. For a JSON report written on graceful
shutdown:

```sh
HOTPATH_OUTPUT_FORMAT=json-pretty HOTPATH_OUTPUT_PATH=/tmp/file-server-profile.json \
just profile /tmp/file-server-local/server.toml
```

For the provisioned Docker dashboard, run these in separate terminals:

```sh
just profiling-up
just profile-grafana /tmp/file-server-local/server.toml
```

Open <http://localhost:3001/d/file-server-hotpath> (user `admin`, password
`local-profiling`). Prometheus is at <http://localhost:9090>. The dashboard shows
route response p95, function time by route, queries per request, SQL duration,
worker function duration, and optional allocations per request. Generate traffic
for at least two scrapes (5 seconds apart) to populate rate-based panels.
`just profiling-down` stops this separate Compose project.

`profile-grafana` exposes the exporter to the Docker bridge and supplies a
matching development bearer token from `profiling/prometheus.yaml`. For a remote
setup, choose your own `HOTPATH_PROMETHEUS_AUTH_TOKEN` and scrape credentials.
The plain `profile` commands keep the exporter on loopback. Hotpath's other local
metrics server defaults to port 6770; `HOTPATH_METRICS_SERVER_OFF=1` disables it.

Interpret measurements with these boundaries:

- Route duration ends at the response head. Download I/O continues to be measured
  as the streaming body is consumed; it is not part of the response-head duration.
- Async function times include waiting and nested calls. Do not add them together
  to calculate request latency.
- `StaticVerifier::verify` includes blocking-pool waiting and Argon2 execution.
  `PasswordHash::verify` measures hashing on the worker itself. Spawned workers
  have no route scope, so use the global function panel to inspect their timings
  and allocations. Route allocation metrics exclude that worker's allocations.
- Annotations do not log arguments or return values. Routes use templates and
  SQL uses bound parameters; filenames, credentials and hashes are not metric labels.
- Debug builds can make Argon2 substantially slower. Compare `profile-debug`
  with `profile` before attributing development response times to the database.

`just check-profile` checks all profiling code. `just test-profile-postgres`
runs the full suite against disposable Docker PostgreSQL, then verifies live
route, SQL, hashing, I/O, allocation and Tokio metrics, exporter authentication,
and JSON report generation during the binary smoke test.
