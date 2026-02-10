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

## License

AGPL-3.0
