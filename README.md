# sigma-telemetry

OpenTelemetry-based observability for the Ryzanstein LLM ecosystem.

## Overview

sigma-telemetry provides structured tracing, metrics, and export for all Ryzanstein operations including inference, model loading, KV cache, speculative decoding, and agent execution.

## Quick Start

```rust
use sigma_telemetry::{SigmaTelemetry, SpanOperation};
use sigma_telemetry::config::TelemetryConfig;

let telemetry = SigmaTelemetry::new(TelemetryConfig::default());

// Start a span (automatically timed)
let mut span = telemetry.start_span("inference", SpanOperation::Inference);
span.set_attribute("model", "bitnet-3b");
span.set_ok();

// Record metrics
telemetry.metrics().increment("ryzanstein.inference.requests");
telemetry.metrics().record_histogram("ryzanstein.inference.latency_ms", 42.5);
telemetry.metrics().set_gauge("ryzanstein.system.gpu_utilization", 85.0);
```

## Architecture

```
Application Code
    ↓
SigmaTelemetry (start_span / metrics)
    ↓
┌─────────┬────────────┬──────────┐
│  Spans  │  Counters  │  Gauges  │
│         │ Histograms │          │
└────┬────┴─────┬──────┴────┬─────┘
     │          │           │
     ▼          ▼           ▼
  Exporter (JSON / OTLP / Stdout)
```

## Well-Known Metrics

| Metric                                   | Type      | Description                     |
| ---------------------------------------- | --------- | ------------------------------- |
| `ryzanstein.inference.requests`          | Counter   | Total inference requests        |
| `ryzanstein.inference.latency_ms`        | Histogram | Inference latency               |
| `ryzanstein.kv_cache.hit_rate`           | Gauge     | KV cache hit rate               |
| `ryzanstein.speculative.acceptance_rate` | Gauge     | Speculative decoding acceptance |
| `ryzanstein.system.gpu_utilization`      | Gauge     | GPU utilization %               |

## Grafana Dashboard

Import [grafana/ryzanstein-dashboard.json](grafana/ryzanstein-dashboard.json) into Grafana 10+.
Set the datasource to **Prometheus** when prompted.

Panels included:

- Inference Requests/sec
- Inference Latency p50 / p95 / p99
- KV Cache Hit Rate
- Speculative Decoding Acceptance Rate
- Active Spans
- GPU / CPU Utilization

## Prometheus /metrics Endpoint

```rust
// Start Prometheus scrape endpoint on port 9091
let handle = telemetry.serve_metrics("127.0.0.1:9091".parse().unwrap());
// Or set TELEMETRY_ADDR=127.0.0.1:9091 before SigmaTelemetry::new() for auto-start.
```

## Rolling Window Aggregation

```rust
use std::time::Duration;
let stats = telemetry.window_stats("ryzanstein.inference.latency_ms", Duration::from_secs(60));
if let Some(s) = stats {
    println!("p99={:.1}ms rate={:.1}/s", s.p99, s.rate);
}
```

## License

AGPL-3.0
