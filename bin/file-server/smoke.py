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
    example = (root / "file_server.example.toml").read_text()
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
            identity = json.dumps({"User": {"id": "alice"}})
            sql = ("INSERT INTO authn_api_tokens (digest, identity) VALUES "
                   f"(decode('{digest}', 'hex'), '{identity}'::jsonb)")
            subprocess.check_call([
                "docker", "compose", "--file", "compose.yaml",
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
