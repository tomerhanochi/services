# File-server development and verification.
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Build the working binary and provisioning helpers.
build:
    cargo build -p file_server --bins --examples

build-release:
    cargo build -p file_server --release --bin file_server

run config="crates/file_server/file_server.example.toml":
    cargo run -p file_server -- {{ quote(config) }}

# Read a password from stdin and print only its independently salted Argon2id hash.
hash-password:
    @cargo run --quiet -p file_server --example hash_password

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
    PGPORT={{ quote(port) }} docker compose --file crates/file_server/compose.yaml up --detach --wait

# Stop the development database while retaining its named volume.
postgres-down:
    docker compose --file crates/file_server/compose.yaml down

# Create a private disposable PostgreSQL container, run everything, and clean it up.
test-postgres port="15433":
    #!/usr/bin/env bash
    set -euo pipefail
    project="file-server-test-$$"
    export PGPORT={{ quote(port) }}
    compose=(docker compose --project-name "$project" --file crates/file_server/compose.yaml)
    cleanup() { "${compose[@]}" down --volumes >/dev/null; }
    trap cleanup EXIT
    "${compose[@]}" up --detach --wait
    export TEST_DATABASE_URL="postgres://file_server:development-only@localhost:$PGPORT/file_server"
    just test-all
    just build
    just smoke

# Test the actual built binary over HTTP, including env-over-TOML config and SIGTERM.
# Requires TEST_DATABASE_URL and a preceding `just build`.
smoke:
    #!/usr/bin/env python3
    """Smoke-test the built binary using a disposable TEST_DATABASE_URL."""
    import base64
    import json
    import os
    from pathlib import Path
    import signal
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
    target = Path(metadata["target_directory"]) / "debug"
    with tempfile.TemporaryDirectory(prefix="file-server-smoke-") as temporary:
        directory = Path(temporary)
        (directory / "users").mkdir()
        (directory / "documents").mkdir()
        (directory / "documents/example.txt").write_text("file-server smoke test\n")
        encoded = subprocess.check_output(
            [str(target / "examples/hash_password")], input=b"smoke-password\n")
        (directory / "users/alice").write_bytes(encoded)
        example = (root / "crates/file_server/file_server.example.toml").read_text()
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
        base = f"http://127.0.0.1:{port}"
        authorization = "Basic " + base64.b64encode(b"alice:smoke-password").decode()
        with (directory / "server.log").open("w+") as log:
            process = subprocess.Popen([str(target / "file_server")], env=environment,
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
                process.send_signal(signal.SIGTERM)
                assert process.wait(timeout=10) == 0
                print("Binary smoke test passed: env-over-TOML config, health, Basic auth, listing, download, rename, graceful shutdown.")
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
