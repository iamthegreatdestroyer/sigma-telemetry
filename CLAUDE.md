# sigma-telemetry — Autonomous Completion Brief

## Project Identity
- **Repo:** `iamthegreatdestroyer/sigma-telemetry`
- **Local path:** `S:\sigma-telemetry`
- **Language:** Rust
- **Castle Layer:** Layer 6 — Operational Intelligence
- **Current completion:** ~65%
- **Mission:** OpenTelemetry-based observability for the Ryzanstein LLM ecosystem — spans, metrics, exports to Prometheus/Grafana/OTLP

## Current State (verified 2026-05-25)
| Component | Status |
|-----------|--------|
| `config.rs` — TelemetryConfig | ✅ Done |
| `error.rs` — TelemetryError | ✅ Done |
| `exporter.rs` — JSON/OTLP/Stdout export | ✅ Done |
| `lib.rs` — SigmaTelemetry public API | ✅ Done |
| `metrics.rs` — Counter/Histogram/Gauge | ✅ Done |
| `ryzanstein_integration.rs` — Ryzanstein-specific metrics | ✅ Done |
| `spans.rs` — SpanOperation tracing | ✅ Done |
| Streaming aggregation (time windows) | ❌ Missing |
| Visualization bridge (Grafana dashboard JSON) | ❌ Missing |
| Integration tests | ❌ Missing |
| Prometheus /metrics HTTP endpoint | ❌ Partial (needs wire-up) |

## Key File Map
```
sigma-telemetry/
├── src/
│   ├── lib.rs                    # SigmaTelemetry, SpanOperation, MetricsCollector
│   ├── config.rs                 # TelemetryConfig (endpoint, service_name, export_format)
│   ├── error.rs                  # TelemetryError
│   ├── exporter.rs               # Exporter trait: JsonExporter, OtlpExporter, StdoutExporter
│   ├── metrics.rs                # MetricsRegistry: increment/histogram/gauge
│   ├── spans.rs                  # SigmaSpan: start_span, set_attribute, set_ok/error
│   └── ryzanstein_integration.rs # Well-known Ryzanstein metric names + helpers
├── tests/                        # Integration tests (sparse)
├── Cargo.toml
└── README.md
```

## What Remains (Final 35%)

### Sprint 1 — Streaming Aggregation (Day 1)
**Goal:** Time-windowed metric aggregation (1s, 10s, 60s rolling windows).

```
@APEX implement aggregation.rs:
  struct RollingWindow {
    window: Duration,
    buckets: VecDeque<(Instant, f64)>,
  }
  impl RollingWindow {
    fn push(&mut self, value: f64)
    fn sum(&self) -> f64       // sum over window
    fn mean(&self) -> f64      // mean over window
    fn percentile(&self, p: f64) -> f64  // p99 etc via t-digest
    fn rate(&self) -> f64      // events per second
  }

Wire into MetricsRegistry: each histogram metric gets a RollingWindow(60s).
Add to SigmaTelemetry API:
  fn window_stats(&self, metric: &str, window: Duration) -> WindowStats

Tests: TestRollingWindowSum, TestRollingWindowPercentile, TestRateCalculation.
```

### Sprint 2 — Prometheus /metrics Endpoint (Day 1–2)
**Goal:** Standard Prometheus text-format HTTP endpoint for Grafana scraping.

```
@APEX wire up a Tokio HTTP server in lib.rs:
  SigmaTelemetry::serve_metrics(&self, addr: SocketAddr) -> JoinHandle<()>

Output format: Prometheus text exposition format (https://prometheus.io/docs/instrumenting/exposition_formats/)
Example:
  # HELP ryzanstein_inference_requests Total inference requests
  # TYPE ryzanstein_inference_requests counter
  ryzanstein_inference_requests 42

Use the existing MetricsRegistry as source. Refresh on each scrape request.
If TELEMETRY_ADDR env var is set, start the server automatically in SigmaTelemetry::new().

Test: TestPrometheusEndpointFormat — start server, GET /metrics, parse response.
```

### Sprint 3 — Grafana Dashboard JSON (Day 2)
**Goal:** Pre-built Grafana dashboard for Ryzanstein LLM monitoring.

```
@SCRIBE create grafana/ryzanstein-dashboard.json with panels for:
  - Inference requests/sec (counter rate)
  - Inference latency p50/p95/p99 (histogram panels)
  - KV cache hit rate (gauge)
  - Speculative decoding acceptance rate (gauge)
  - GPU/CPU utilization (gauge)
  - Active spans (derived from trace data)

Dashboard should use datasource: "Prometheus" with variable $interval=60s.
Add to README.md: "Import grafana/ryzanstein-dashboard.json into Grafana 10+"
```

### Sprint 4 — Integration Tests + Build (Day 3)
```
@CORE write tests/integration_test.rs:
  test_full_span_lifecycle:
    1. Create SigmaTelemetry with StdoutExporter
    2. Start span "test_op" with operation=Inference
    3. Increment counter "test.requests" 
    4. Record histogram "test.latency" with 42.0
    5. Set span ok, drop span
    6. Assert: exporter received 1 span, 1 counter event, 1 histogram event

  test_prometheus_format:
    Start metrics server on 127.0.0.1:19090
    Record metrics
    GET http://127.0.0.1:19090/metrics
    Assert response contains "ryzanstein_" prefixed metrics

Run: cargo test -- --test-threads=1 (avoid port conflicts)
cargo clippy -- -D warnings
cargo build --release
git tag v0.2.0 && git push origin v0.2.0
```

## Done Criteria (all must pass)
- [x] `cargo test` passes — zero failures
- [x] `RollingWindow` aggregation: p99 and rate calculations correct
- [ ] Prometheus `/metrics` endpoint serves valid text-format output
- [x] Grafana dashboard JSON committed at `grafana/ryzanstein-dashboard.json`
- [x] `cargo clippy -- -D warnings` clean
- [x] `cargo build --release` succeeds
- [ ] `v0.2.0` tag pushed

## Completion Signal
```bash
git tag v0.2.0 && git push origin v0.2.0
```

## Critical Rules
1. **Non-blocking metrics** — MetricsRegistry operations must never block the caller; use lock-free or bounded channels
2. **Prometheus format compliance** — follow the spec exactly; Grafana won't parse invalid format
3. **Graceful shutdown** — metrics server must shut down cleanly on SIGTERM; no orphan threads
4. **No panics in prod** — all public API functions return Result<T, TelemetryError>; never unwrap in library code
