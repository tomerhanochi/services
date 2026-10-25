# File-server development and verification.
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Build the working binary and examples.
build:
    cargo build -p file-server --bins --examples

build-release:
    cargo build -p file-server --release --bin file-server

run config="crates/file-server/file_server.example.toml":
    cargo run -p file-server -- {{ quote(config) }}

build-profile:
    cargo build --release -p file-server --bins --examples --features hotpath-prometheus

# Profile optimized code; Ctrl-C flushes the function/SQL/route report.
profile config="crates/file-server/file_server.example.toml":
    cargo run --release -p file-server --features hotpath-prometheus -- {{ quote(config) }}

# Match `just run` optimization settings when investigating development latency.
profile-debug config="crates/file-server/file_server.example.toml":
    cargo run -p file-server --features hotpath-prometheus -- {{ quote(config) }}

# Allocation tracking adds overhead; enable it separately from timing runs.
profile-alloc config="crates/file-server/file_server.example.toml":
    cargo run --release -p file-server --features hotpath-prometheus,hotpath-alloc -- {{ quote(config) }}

# The local Docker scraper uses the matching development-only bearer token.
profile-grafana config="crates/file-server/file_server.example.toml":
    HOTPATH_PROMETHEUS_HOST=0.0.0.0 HOTPATH_PROMETHEUS_AUTH_TOKEN=file-server-local-profiling just profile {{ quote(config) }}

# Grafana :3001 (admin / local-profiling), Prometheus :9090, both localhost-only.
profiling-up:
    docker compose --file crates/file-server/profiling/compose.yaml up --detach --wait

profiling-down:
    docker compose --file crates/file-server/profiling/compose.yaml down

check-profile:
    cargo check --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc

test-profile:
    cargo test --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc

# Use the Docker integration workflow with profiling and assert live telemetry.
test-profile-postgres port="15434":
    SMOKE_HOTPATH=1 just test-postgres {{ quote(port) }}

fmt:
    cargo fmt --all

check:
    cargo fmt --all --check
    cargo check --workspace --all-targets

# No database required; database integration tests are explicitly ignored.
test:
    cargo test --workspace --all-targets

# Requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database.
test-all:
    cargo test --workspace --all-targets -- --include-ignored --test-threads=1

postgres-up port="15432":
    PGPORT={{ quote(port) }} docker compose --file crates/file-server/compose.yaml up --detach --wait

# Stop the development database while retaining its named volume.
postgres-down:
    docker compose --file crates/file-server/compose.yaml down

# Create a private disposable PostgreSQL container, run everything, and clean it up.
test-postgres port="15433":
    #!/usr/bin/env bash
    set -euo pipefail
    project="file-server-test-$$"
    export COMPOSE_PROJECT_NAME="$project"
    export PGPORT={{ quote(port) }}
    compose=(docker compose --project-name "$project" --file crates/file-server/compose.yaml)
    cleanup() { "${compose[@]}" down --volumes >/dev/null; }
    trap cleanup EXIT
    "${compose[@]}" up --detach --wait
    export TEST_DATABASE_URL="postgres://file_server:development-only@localhost:$PGPORT/file_server"
    if [[ "${SMOKE_HOTPATH:-}" == 1 ]]; then
        cargo test --workspace --all-targets --features file-server/hotpath-prometheus,file-server/hotpath-alloc -- --include-ignored --test-threads=1
        cargo build -p file-server --bins --examples --features hotpath-prometheus,hotpath-alloc
    else
        just test-all
        just build
    fi
    just smoke

# Test the actual built binary over HTTP, including env-over-TOML config and SIGTERM.
# Requires TEST_DATABASE_URL and a preceding `just build`.
smoke:
    #!/usr/bin/env python3
    """Smoke-test the built binary using a disposable TEST_DATABASE_URL."""
    import hashlib
    import json
    import os
    from pathlib import Path
    import signal
    import secrets
    import socket
    import subprocess
    import tempfile
    import time
    import urllib.error
    import urllib.parse
    import urllib.request

    root = Path.cwd()
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--format-version=1", "--no-deps"], cwd=root))
    target = Path(metadata["target_directory"]) / os.environ.get("SMOKE_BUILD_PROFILE", "debug")
    with tempfile.TemporaryDirectory(prefix="file-server-smoke-") as temporary:
        directory = Path(temporary)
        (directory / "documents").mkdir()
        (directory / "documents/example.txt").write_text("file-server smoke test\n")
        example = (root / "crates/file-server/file_server.example.toml").read_text()
        example += f"\n[auth.token]\ndatabase_url = {os.environ['TEST_DATABASE_URL']!r}\n"
        # The invalid file value must be overridden by the environment.
        example = example.replace(
            "postgres://file_server:development-only@localhost:15432/file_server",
            "postgres://invalid.invalid/never-connect")
        (directory / "server.toml").write_text(example)
        with socket.socket() as available:
            available.bind(("127.0.0.1", 0))
            port = available.getsockname()[1]
        environment = os.environ.copy()
        environment.update({
            "FILE_SERVER_CONFIG": str(directory / "server.toml"),
            "FILE_SERVER_DATABASE__URL": os.environ["TEST_DATABASE_URL"],
            "FILE_SERVER_LISTEN": f"127.0.0.1:{port}",
            "FILE_SERVER_PUBLIC_URL": f"http://127.0.0.1:{port}",
        })
        profiling = os.environ.get("SMOKE_HOTPATH") == "1"
        if profiling:
            # Private exporter port; disable the other profiler listeners for isolation.
            with socket.socket() as available:
                available.bind(("127.0.0.1", 0))
                metrics_port = available.getsockname()[1]
            environment.update({
                "HOTPATH_PROMETHEUS_HOST": "127.0.0.1",
                "HOTPATH_PROMETHEUS_PORT": str(metrics_port),
                "HOTPATH_PROMETHEUS_AUTH_TOKEN": "smoke-metrics-token",
                "HOTPATH_METRICS_SERVER_OFF": "1",
                "HOTPATH_OUTPUT_FORMAT": "json",
                "HOTPATH_OUTPUT_PATH": str(directory / "profile.json"),
            })
        base = f"http://127.0.0.1:{port}"
        with (directory / "server.log").open("w+") as log:
            process = subprocess.Popen([str(target / "file-server")], env=environment,
                                       stdout=log, stderr=subprocess.STDOUT)
            try:
                for attempt in range(100):
                    if process.poll() is not None:
                        log.seek(0)
                        raise RuntimeError(log.read())
                    try:
                        with urllib.request.urlopen(base + "/healthz", timeout=1) as response:
                            assert response.read() == b"ok"
                        break
                    except (urllib.error.URLError, TimeoutError):
                        time.sleep(0.1)
                else:
                    raise RuntimeError("binary did not become ready")
                token = "smoke-" + secrets.token_urlsafe(32)
                digest = hashlib.sha256(token.encode()).hexdigest()
                sql = ("INSERT INTO authn_api_tokens (digest, identity) VALUES "
                       f"(decode('{digest}', 'hex'), '{{\"User\":{{\"id\":\"alice\"}}}}'::jsonb)")
                subprocess.check_call([
                    "docker", "compose", "--file", "crates/file-server/compose.yaml",
                    "exec", "-T", "postgres", "psql", "-U", "file_server", "-d", "file_server", "-c", sql,
                ], cwd=root, stdout=subprocess.DEVNULL)
                authorization = "Bearer " + token
                def get(path):
                    request = urllib.request.Request(base + path,
                                                     headers={"Authorization": authorization})
                    return urllib.request.urlopen(request, timeout=5)
                with get("/m/documents/") as response:
                    assert response.status == 200
                    assert b"example.txt" in response.read()
                with get("/m/documents/example.txt") as response:
                    assert response.read() == b"file-server smoke test\n"
                request = urllib.request.Request(
                    base + "/rename/documents/example.txt",
                    data=urllib.parse.urlencode({"name": "renamed.txt"}).encode(),
                    headers={"Authorization": authorization})
                with urllib.request.urlopen(request, timeout=5) as response:
                    assert response.status == 200  # follows the 303 to the listing
                assert (directory / "documents/renamed.txt").read_text() == "file-server smoke test\n"
                if profiling:
                    for attempt in range(50):
                        request = urllib.request.Request(
                            f"http://127.0.0.1:{metrics_port}/metrics",
                            headers={"Authorization": "Bearer smoke-metrics-token"})
                        with urllib.request.urlopen(request, timeout=5) as response:
                            metrics = response.read().decode()
                        if all(name in metrics for name in [
                            "hotpath_server_sql_calls_total", "hotpath_io_bytes_total",
                            "TokenVerifier::verify", "PostgresAccess::capabilities",
                            "hotpath_function_alloc_bytes_total", "hotpath_tokio_workers",
                        ]):
                            break
                        time.sleep(0.1)
                    else:
                        raise AssertionError("Expected route, SQL, token, I/O, allocation and runtime telemetry")
                    assert 'route="GET /m/{mount}/"' in metrics
                    sql_counts = [line for line in metrics.splitlines()
                                  if line.startswith("hotpath_server_sql_calls_total{")
                                  and 'route="GET /m/{mount}/"' in line]
                    assert sql_counts and any(float(line.rsplit(" ", 1)[1]) > 0 for line in sql_counts)
                    assert any(line.startswith("hotpath_sql_queries_total{")
                               and 'route="GET /m/{mount}/"' in line
                               and "PostgresAccess::capabilities" in line
                               for line in metrics.splitlines())
                    try:
                        urllib.request.urlopen(f"http://127.0.0.1:{metrics_port}/metrics", timeout=5)
                    except urllib.error.HTTPError as error:
                        assert error.code == 401
                    else:
                        raise AssertionError("Exporter accepted an unauthenticated scrape")
                    print("Profiling smoke test passed: route, SQL, token auth, I/O, allocations, runtime, exporter auth.")
                process.send_signal(signal.SIGTERM)
                assert process.wait(timeout=10) == 0
                if profiling:
                    report = json.loads((directory / "profile.json").read_text())
                    assert report
                    print("Profiling JSON report flushed on graceful shutdown.")
                print("Binary smoke test passed: env-over-TOML config, health, bearer auth, listing, download, rename, graceful shutdown.")
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()

# Measure release latency, allocations and RSS against disposable Docker PostgreSQL.
measure-release output="/tmp/file-server-measurement" samples="30" port="15435":
    #!/usr/bin/env bash
    set -euo pipefail
    project="file-server-measure-$$"
    export COMPOSE_PROJECT_NAME="$project"
    export PGPORT={{ quote(port) }}
    compose=(docker compose --project-name "$project" --file crates/file-server/compose.yaml)
    trap '"${compose[@]}" down --volumes >/dev/null' EXIT
    "${compose[@]}" up --detach --wait
    export TEST_DATABASE_URL="postgres://file_server:development-only@localhost:$PGPORT/file_server"
    if [[ "${MEASURE_INSTRUMENTATION:-alloc}" == off ]]; then
        just build-release
    else
        cargo build --release -p file-server --bins --examples --features hotpath-prometheus,hotpath-alloc
    fi
    just _measure-release {{ quote(output) }} {{ quote(samples) }}

[private]
_measure-release output samples:
    #!/usr/bin/env python3
    import hashlib
    import http.client
    import concurrent.futures
    import gzip
    import threading
    import json
    import math
    import os
    from pathlib import Path
    import re
    import signal
    import secrets
    import socket
    import statistics
    import subprocess
    import tempfile
    import time
    import urllib.request

    output = Path({{ quote(output) }}).resolve()
    output.mkdir(parents=True, exist_ok=True)
    profiling = os.environ.get("MEASURE_INSTRUMENTATION", "alloc") != "off"
    samples = int({{ quote(samples) }})
    assert samples >= 10, "Use at least ten samples per scenario"
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--format-version=1", "--no-deps"]))
    target = Path(metadata["target_directory"]) / "release"
    def port():
        with socket.socket() as available:
            available.bind(("127.0.0.1", 0))
            return available.getsockname()[1]
    app_port, metrics_port = port(), port()
    def metrics():
        if not profiling:
            rss = int(subprocess.check_output(["ps", "-o", "rss=", "-p", str(process.pid)])) * 1024
            return {("hotpath_rss_bytes", ()): rss}, ""
        with urllib.request.urlopen(f"http://127.0.0.1:{metrics_port}/metrics", timeout=5) as response:
            raw = response.read().decode()
        parsed = {}
        for line in raw.splitlines():
            match = re.fullmatch(r'(\w+)(?:\{(.*)\})? ([^ ]+)', line)
            if match:
                labels = tuple(sorted((m[0], json.loads(m[1])) for m in
                    re.findall(r'(\w+)=("(?:[^"\\]|\\.)*")', match[2] or "")))
                parsed[match[1], labels] = float(match[3])
        return parsed, raw
    def value(data, name):
        return sum(v for (n, labels), v in data.items() if n == name)
    def delta(before, after, name):
        return value(after, name) - value(before, name)
    def stats(values):
        ordered = sorted(values)
        return {"mean_ms": statistics.mean(values), "median_ms": statistics.median(values),
                "p95_ms": ordered[math.ceil(len(ordered) * .95) - 1]}
    with tempfile.TemporaryDirectory(prefix="file-server-measure-") as temporary:
        fixture = Path(temporary)
        documents = fixture / "documents"
        documents.mkdir()
        for count in (1, 100, 1000):
            directory = documents / str(count)
            directory.mkdir()
            for i in range(count):
                (directory / f"entry-{i:04}.txt").write_text("measurement fixture\n")
        (documents / "download.bin").write_bytes(b"x" * (1024 * 1024))
        config = Path("crates/file-server/file_server.example.toml").read_text()
        config += f"\n[auth.token]\ndatabase_url = {os.environ['TEST_DATABASE_URL']!r}\n"
        (fixture / "server.toml").write_text(config)
        environment = {k: v for k, v in os.environ.items()
                       if not k.startswith(("FILE_SERVER_", "HOTPATH_"))}
        environment.update({
            "FILE_SERVER_LISTEN": f"127.0.0.1:{app_port}",
            "FILE_SERVER_PUBLIC_URL": f"http://127.0.0.1:{app_port}",
            "FILE_SERVER_DATABASE__URL": os.environ["TEST_DATABASE_URL"],
            "HOTPATH_PROMETHEUS_PORT": str(metrics_port),
            "HOTPATH_METRICS_SERVER_OFF": "1", "HOTPATH_OUTPUT_FORMAT": "json-pretty",
            "HOTPATH_OUTPUT_PATH": str(output / "hotpath.json"),
            "RUST_LOG": "info",
        })
        results = {"build": "release", "allocation_tracking": profiling, "samples_per_scenario": samples,
                   "authentication": "database-backed API token", "scenarios": []}
        with (output / "server.log").open("w+") as log:
            process = subprocess.Popen([str(target / "file-server"), str(fixture / "server.toml")],
                                       env=environment, stdout=log, stderr=subprocess.STDOUT)
            try:
                for attempt in range(100):
                    try:
                        urllib.request.urlopen(f"http://127.0.0.1:{app_port}/healthz", timeout=1).close()
                        metrics()
                        break
                    except OSError:
                        if process.poll() is not None:
                            log.seek(0)
                            raise RuntimeError(log.read())
                        time.sleep(.1)
                else:
                    raise RuntimeError("Server not ready")
                token = "measure-" + secrets.token_urlsafe(32)
                digest = hashlib.sha256(token.encode()).hexdigest()
                sql = ("INSERT INTO authn_api_tokens (digest, identity) VALUES "
                       f"(decode('{digest}', 'hex'), '{{\"User\":{{\"id\":\"alice\"}}}}'::jsonb)")
                subprocess.check_call([
                    "docker", "compose", "--file", "crates/file-server/compose.yaml",
                    "exec", "-T", "postgres", "psql", "-U", "file_server", "-d", "file_server", "-c", sql,
                ], stdout=subprocess.DEVNULL)
                authorization = "Bearer " + token
                time.sleep(.5)
                initial, _ = metrics()
                results["startup_rss_bytes"] = value(initial, "hotpath_rss_bytes")
                connection = http.client.HTTPConnection("127.0.0.1", app_port, timeout=30)
                for name, path, auth, concurrency in [
                    ("health", "/healthz", False, 1), ("anonymous_home", "/", False, 1),
                    ("authenticated_home", "/", True, 1),
                    ("listing_1", "/m/documents/1/", True, 1),
                    ("listing_100", "/m/documents/100/", True, 1),
                    ("listing_1000", "/m/documents/1000/", True, 1),
                    ("download_1MiB", "/m/documents/download.bin", True, 1),
                    ("listing_1_concurrent_8", "/m/documents/1/", True, 8),
                ]:
                    headers = {"Accept-Encoding": "gzip"}
                    if auth:
                        headers["Authorization"] = authorization
                    local = threading.local()
                    connections = []
                    def request(_=None):
                        if concurrency == 1:
                            client = connection
                        else:
                            if not hasattr(local, "client"):
                                local.client = http.client.HTTPConnection("127.0.0.1", app_port, timeout=30)
                                connections.append(local.client)
                            client = local.client
                        start = time.perf_counter()
                        client.request("GET", path, headers=headers)
                        response = client.getresponse()
                        head = time.perf_counter()
                        assert response.status == 200, (path, response.status)
                        body = response.read()
                        end = time.perf_counter()
                        plain_size = len(gzip.decompress(body)) if response.getheader("Content-Encoding") == "gzip" else len(body)
                        return (head - start) * 1000, (end - start) * 1000, len(body), plain_size
                    for _ in range(3):
                        request()
                    time.sleep(.2)
                    before, _ = metrics()
                    if concurrency == 1:
                        timings = [request() for _ in range(samples)]
                    else:
                        with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as pool:
                            timings = list(pool.map(request, range(samples)))
                        for client in connections:
                            client.close()
                    time.sleep(.3)
                    after, raw = metrics()
                    (output / f"{name}.prom").write_text(raw)
                    functions = {}
                    for (metric, labels), calls in after.items():
                        if metric != "hotpath_function_calls_total":
                            continue
                        calls -= before.get((metric, labels), 0)
                        if calls <= 0:
                            continue
                        def change(metric):
                            return after.get((metric, labels), 0) - before.get((metric, labels), 0)
                        function = dict(labels)["function"]
                        functions[function] = {
                            "calls_per_request": calls / samples,
                            "mean_ms": change("hotpath_function_duration_seconds_sum") * 1000 / calls,
                            "allocated_bytes_per_call": change("hotpath_function_alloc_bytes_total") / calls,
                            "allocations_per_call": change("hotpath_function_alloc_count_total") / calls,
                        }
                    result = {"name": name, "path": path, "concurrency": concurrency, "head": stats([t[0] for t in timings]),
                              "full_body": stats([t[1] for t in timings]), "response_bytes": timings[-1][2], "uncompressed_response_bytes": timings[-1][3],
                              "head_samples_ms": [t[0] for t in timings],
                              "route_allocated_bytes_per_request": delta(before, after, "hotpath_server_alloc_bytes_total") / samples,
                              "route_allocations_per_request": delta(before, after, "hotpath_server_alloc_count_total") / samples,
                              "sql_queries_per_request": delta(before, after, "hotpath_sql_queries_total") / samples,
                              "sql_total_ms_per_request": delta(before, after, "hotpath_sql_duration_seconds_sum") * 1000 / samples,
                              "rss_bytes_after": value(after, "hotpath_rss_bytes"),
                              "process_allocated_bytes_per_request": delta(before, after, "hotpath_alloc_bytes_total") / samples,
                              "process_deallocated_bytes_per_request": delta(before, after, "hotpath_dealloc_bytes_total") / samples,
                              "functions": functions}
                    if not profiling:
                        for key in ("route_allocated_bytes_per_request", "route_allocations_per_request",
                                    "sql_queries_per_request", "sql_total_ms_per_request",
                                    "process_allocated_bytes_per_request", "process_deallocated_bytes_per_request"):
                            result[key] = None
                    results["scenarios"].append(result)
                    print(f"{name}: {result['head']['mean_ms']:.2f} ms mean head, {result['head']['p95_ms']:.2f} ms p95; "
                          f"{str(result['route_allocated_bytes_per_request'])} route bytes/request; "
                          f"{result['rss_bytes_after'] / 1024**2:.1f} MiB RSS", flush=True)
                connection.close()
                time.sleep(2)
                idle, _ = metrics()
                results["idle_rss_bytes_after_workload"] = value(idle, "hotpath_rss_bytes")
                (output / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
            finally:
                process.send_signal(signal.SIGTERM)
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        assert process.returncode == 0
    print(f"Saved measurements to {output}" + (" (including hotpath report)" if profiling else ""))

# Render editor/viewer listings and count repeated SVG markup without a database.
inspect-html output="/tmp/file-server-html":
    cargo run --release -p file-server-views --example html_size -- {{ quote(output) }}
