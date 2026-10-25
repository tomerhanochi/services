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
import sys
import tempfile
import time
import urllib.request

output = Path(sys.argv[1]).resolve()
output.mkdir(parents=True, exist_ok=True)
profiling = os.environ.get("MEASURE_INSTRUMENTATION", "alloc") != "off"
samples = int(sys.argv[2])
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
    config = Path("file_server.example.toml").read_text()
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
            identity = json.dumps({"User": {"id": "alice"}})
            sql = ("INSERT INTO authn_api_tokens (digest, identity) VALUES "
                   f"(decode('{digest}', 'hex'), '{identity}'::jsonb)")
            subprocess.check_call([
                    "docker", "compose", "--file", "compose.yaml",
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
