# Release request timing and memory measurements

Measured 2026-10-16 on this macOS ARM64 development machine, using local
filesystem fixtures and PostgreSQL 18 in a disposable Docker container. The
application source is commit `be4dcd3`; no application behavior was changed.

Each scenario has 3 warm-up requests and 60 measured requests. Requests use
HTTP keep-alive and accept gzip. Scenarios run in the order below in one server
process per build. Normal scenarios have one client; the last has up to eight.
The first run uses release + hotpath timing, future, SQL, route, I/O, thread and
allocation profiling. The control uses release with profiling disabled.
Timings are measured by the HTTP client until response headers arrive. RSS is
sampled after each scenario, not an isolated per-request memory measurement.

| Scenario | Profiled mean / p95 (ms) | Control mean / p95 (ms) | Profiled RSS (MiB) | Control RSS (MiB) |
|---|---:|---:|---:|---:|
| health | 0.18 / 0.24 | 0.22 / 0.25 | 21.6 | 10.9 |
| anonymous_home | 0.21 / 0.30 | 0.27 / 0.39 | 27.3 | 11.7 |
| authenticated_home | 12.96 / 14.39 | 13.31 / 15.00 | 53.2 | 31.0 |
| listing_1 | 13.96 / 15.06 | 14.16 / 15.32 | 59.5 | 31.2 |
| listing_100 | 17.90 / 18.96 | 18.21 / 19.66 | 61.3 | 31.8 |
| listing_1000 | 53.34 / 54.58 | 51.85 / 52.62 | 261.0 | 229.8 |
| download_1MiB | 14.02 / 15.16 | 14.13 / 15.34 | 264.2 | 229.8 |
| listing_1_concurrent_8 | 24.92 / 37.87 | 26.16 / 44.62 | 377.3 | 342.4 |

## Where the time goes

- Basic authentication verifies Argon2 on every authenticated request. In the
  serial scenarios, `PasswordHash::verify` averages 11.3–11.6 ms, accounting
  for most of the 13–14 ms small-page response. The hash uses
  `argon2id`, `m=19456,t=2,p=1`. Credential-file lookup adds about 0.1 ms.
- A directory response executes two access queries: one before metadata and
  another before listing. SQL execution totals about 1.1–1.3 ms per request.
  The enclosing access-store calls total about 2.1–2.5 ms, including pool and
  client-side work. These are nested timings; do not add SQL time again.
- Listing 1,000 entries spends 32.6 ms inside the filesystem listing. It makes
  1,001 metadata calls and 1,002 path-resolution calls per request, including
  the initial directory check. Each entry is resolved and inspected serially.
- HTML serialization is about 0.81 ms for 1,000 entries. The list service adds
  roughly 2.4 ms beyond filesystem listing, including its access check and
  sorting, based on the difference between the measured enclosing calls.
- Under eight concurrent clients, password verification averages 20.4 ms,
  versus about 11.3 ms serially, consistent with CPU/memory contention. The
  async verifier averages 20.7 ms, so blocking-pool wait is not the main gap.
- The synthetic 1 MiB download completes in 16.9 ms on average with profiling,
  versus 14.0 ms to headers. Its repeated-byte payload compresses to 1,051
  bytes, so this is not a general network-throughput benchmark.

## Allocations and resident memory

Allocation bytes are cumulative allocation traffic, including reallocations;
they are not simultaneously live memory. Route metrics exclude spawned workers.

| Scenario | Allocated on route per request (KiB) | Argon2 worker per request (MiB) | HTML serializer per request (KiB) |
|---|---:|---:|---:|
| health | 10.9 | 0.0 | 0.0 |
| anonymous_home | 338.4 | 0.0 | 1.5 |
| authenticated_home | 349.6 | 19.0 | 4.5 |
| listing_1 | 391.0 | 19.0 | 12.0 |
| listing_100 | 1460.1 | 19.0 | 921.7 |
| listing_1000 | 11226.2 | 19.0 | 9194.7 |
| download_1MiB | 378.2 | 19.0 | 0.0 |
| listing_1_concurrent_8 | 394.1 | 19.0 | 12.0 |

- Every password verification allocates one 19 MiB Argon2 working buffer,
  including in the concurrent scenario. Eight simultaneous verifications can
  therefore require 152 MiB for those buffers alone. They are temporary.
- The 1,000-entry response contains 3,267,820 bytes of uncompressed HTML
  (about 3.12 MiB), versus about 45 KiB gzipped. `htmlb::to_html` records two
  allocations totaling 9,415,422 bytes (8.98 MiB). It creates a String with an
  estimated capacity and then appends HTML, so buffer growth is a useful
  follow-up target. The view repeats icons and action-sheet markup per entry.
- Sorting the 1,000 entries accounts for most of the service wrapper's
  228,288 allocated bytes and 21,537 allocations per request. Path-resolution
  and metadata calls add many small allocations.
- Startup RSS was 21.2 MiB profiled and 10.7 MiB in the control.
  After the entire workload and two idle seconds it remained 377.3 MiB
  profiled and 342.4 MiB in the control. The high RSS is therefore
  not solely profiling overhead. This short test cannot distinguish allocator
  retention from a memory leak; RSS is not a live-heap measurement.
- The initial independent 30-sample profiling run produced 14.1 ms for a
  one-entry listing and 53.6 ms for 1,000 entries, consistent with the repeat.
  Its RSS after the large listing was 167.7 MiB, versus 261.0 MiB after the
  60-sample listing. Retained memory warrants a longer steady-state study.

## Implication for the reported 268 ms

`just run` invokes plain `cargo run`, which uses the unoptimized development
build. These release runs did not reproduce 268 ms. Development-build overhead
in CPU-heavy password hashing is plausible, but was not measured because only
release builds were requested.

The next useful optimization targets are avoiding repeated password hashing
through an authenticated session design, reducing serial per-entry filesystem
work, and reducing HTML buffer growth/markup volume. Do not weaken the password
hash parameters merely to reduce request latency.

## Reproduce

```sh
just measure-release /tmp/file-server-measurement-repeat 60
MEASURE_INSTRUMENTATION=off just measure-release /tmp/file-server-measurement-control 60
```

The recipe builds release, creates disposable fixtures and PostgreSQL, runs the
workload, and cleans up the server/container/volume. Use its third argument to
change the database port from 15435. Reports include `summary.json`, raw
per-scenario Prometheus snapshots, `server.log`, and (when profiling is enabled)
`hotpath.json`. Summary JSON includes individual response-header timings.
The profiled summaries use hotpath RSS samples; the control reads RSS using
`ps`. The database and Python load generator are separate processes and are
excluded from application RSS.
