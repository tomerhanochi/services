# Load profiling and measurement commands.
mod profiling
set shell := ["bash", "-euo", "pipefail", "-c"]

# Build the file-server binary and examples.
build:
    cargo build -p file-server --bins --examples

# Build the optimized file-server binary.
build-release:
    cargo build -p file-server --release --bin file-server

# Run file-server with a TOML configuration.
run config="file_server.example.toml":
    cargo run -p file-server -- {{ quote(config) }}

# Start the local HTTPS stack and run the server with its development config.
dev: (dev-up)
    RUST_LOG="${RUST_LOG:-debug}" cargo run -p file-server -- config.dev.toml

# Start the local HTTPS development dependencies.
dev-up:
    docker compose --file compose.dev.yaml up --detach --wait postgres hydra consent https
    docker compose --file compose.dev.yaml run --rm hydra-client

# Stop the local HTTPS development dependencies.
dev-down:
    docker compose --file compose.dev.yaml down

# Profile optimized code; Ctrl-C flushes the function/SQL/route report.
profile config="file_server.example.toml":
    cargo run --release -p file-server --features hotpath-prometheus -- {{ quote(config) }}

# Match `run` optimization settings when investigating development latency.
profile-debug config="file_server.example.toml":
    cargo run -p file-server --features hotpath-prometheus -- {{ quote(config) }}

# Allocation tracking adds overhead; enable it separately from timing runs.
profile-alloc config="file_server.example.toml":
    cargo run --release -p file-server --features hotpath-prometheus,hotpath-alloc -- {{ quote(config) }}

# The local Docker scraper uses the matching development-only bearer token.
profile-grafana config="file_server.example.toml":
    HOTPATH_PROMETHEUS_HOST=0.0.0.0 HOTPATH_PROMETHEUS_AUTH_TOKEN=file-server-local-profiling just bin file-server profile {{ quote(config) }}

# Grafana :3001 (admin / local-profiling), Prometheus :9090, both localhost-only.
profiling-up:
    docker compose --file profiling/compose.yaml up --detach --wait

# Stop the local profiling dashboard services.
profiling-down:
    docker compose --file profiling/compose.yaml down

# Check profiling-enabled code across the workspace.
check-profile:
    cargo check --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc

# Test profiling-enabled code across the workspace.
test-profile:
    cargo test --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc

# Use the Docker integration workflow with profiling and assert live telemetry.
test-profile-postgres port="15434":
    SMOKE_HOTPATH=1 just bin file-server test-postgres {{ quote(port) }}

# Start the example PostgreSQL database.
postgres-up port="15432":
    PGPORT={{ quote(port) }} docker compose --file compose.yaml up --detach --wait

# Stop the development database while retaining its named volume.
postgres-down:
    docker compose --file compose.yaml down

# Create a private disposable PostgreSQL container, run everything, and clean it up.
test-postgres port="15433":
    #!/usr/bin/env bash
    set -euo pipefail
    project="file-server-test-$$"
    export COMPOSE_PROJECT_NAME="$project"
    export PGPORT={{ quote(port) }}
    compose=(docker compose --project-name "$project" --file compose.yaml)
    cleanup() { "${compose[@]}" down --volumes >/dev/null; }
    trap cleanup EXIT
    "${compose[@]}" up --detach --wait
    export TEST_DATABASE_URL="postgres://file_server:development-only@localhost:$PGPORT/file_server"
    if [[ "${SMOKE_HOTPATH:-}" == 1 ]]; then
        cargo test --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc -- --include-ignored --test-threads=1
        cargo build -p file-server --bins --examples --features hotpath-prometheus,hotpath-alloc
    else
        just test-all
        just bin file-server build
    fi
    python3 smoke.py

# Render editor/viewer listings and count repeated SVG markup without a database.
inspect-html output="/tmp/file-server-html":
    cargo run --release -p file-server-views --example html_size -- {{ quote(output) }}
