set shell := ["bash", "-euo", "pipefail", "-c"]
set working-directory := ".."

# Measure release latency, allocations and RSS against disposable Docker PostgreSQL.
measure-release output="/tmp/file-server-measurement" samples="30" port="15435":
    #!/usr/bin/env bash
    set -euo pipefail
    project="file-server-measure-$$"
    export COMPOSE_PROJECT_NAME="$project"
    export PGPORT={{ quote(port) }}
    compose=(docker compose --project-name "$project" --file compose.yaml)
    trap '"${compose[@]}" down --volumes >/dev/null' EXIT
    "${compose[@]}" up --detach --wait
    export TEST_DATABASE_URL="postgres://file_server:development-only@localhost:$PGPORT/file_server"
    if [[ "${MEASURE_INSTRUMENTATION:-alloc}" == off ]]; then
        just bin file-server build-release
    else
        cargo build --release -p file-server --bins --examples --features hotpath-prometheus,hotpath-alloc
    fi
    python3 profiling/measure_release.py {{ quote(output) }} {{ quote(samples) }}
