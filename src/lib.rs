//! # sigma-telemetry
//!
//! OpenTelemetry-based observability for the Ryzanstein LLM ecosystem.
//! Provides structured tracing, metrics, and log correlation for inference
//! pipelines, model loading, and agent orchestration.

pub mod config;
pub mod error;
pub mod metrics;
pub mod spans;
pub mod exporter;
pub mod ryzanstein_integration;

use std::sync::Arc;
use std::time::{Duration, Instant};
use config::TelemetryConfig;
use error::TelemetryError;

/// Core telemetry system for Ryzanstein
pub struct SigmaTelemetry {
    config: TelemetryConfig,
    metrics: MetricsCollector,
    active_spans: std::sync::Mutex<Vec<SpanRecord>>,
}

/// Recorded span information
#[derive(Debug, Clone)]
pub struct SpanRecord {
    pub name: String,
    pub service: String,
    pub operation: SpanOperation,
    pub start_time: std::time::SystemTime,
    pub duration: Option<Duration>,
    pub attributes: Vec<(String, String)>,
    pub status: SpanStatus,
}

/// Well-known span operations for Ryzanstein
#[derive(Debug, Clone, PartialEq)]
pub enum SpanOperation {
    ModelLoad,
    Inference,
    TokenGeneration,
    KvCacheOp,
    SpeculativeDraft,
    SpeculativeVerify,
    EmbeddingEncode,
    AgentExecute,
    VaultStore,
    VaultRetrieve,
    Custom(String),
}

impl std::fmt::Display for SpanOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanOperation::ModelLoad => write!(f, "model.load"),
            SpanOperation::Inference => write!(f, "inference"),
            SpanOperation::TokenGeneration => write!(f, "token.generation"),
            SpanOperation::KvCacheOp => write!(f, "kv_cache.op"),
            SpanOperation::SpeculativeDraft => write!(f, "speculative.draft"),
            SpanOperation::SpeculativeVerify => write!(f, "speculative.verify"),
            SpanOperation::EmbeddingEncode => write!(f, "embedding.encode"),
            SpanOperation::AgentExecute => write!(f, "agent.execute"),
            SpanOperation::VaultStore => write!(f, "vault.store"),
            SpanOperation::VaultRetrieve => write!(f, "vault.retrieve"),
            SpanOperation::Custom(name) => write!(f, "custom.{}", name),
        }
    }
}

/// Span status
#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Ok,
    Error(String),
    Unset,
}

/// Metrics collector
pub struct MetricsCollector {
    counters: std::sync::Mutex<std::collections::HashMap<String, u64>>,
    histograms: std::sync::Mutex<std::collections::HashMap<String, Vec<f64>>>,
    gauges: std::sync::Mutex<std::collections::HashMap<String, f64>>,
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            counters: std::sync::Mutex::new(std::collections::HashMap::new()),
            histograms: std::sync::Mutex::new(std::collections::HashMap::new()),
            gauges: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Increment a counter by 1
    pub fn increment(&self, name: &str) {
        self.increment_by(name, 1);
    }

    /// Increment a counter by a specific amount
    pub fn increment_by(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    /// Record a histogram value (e.g., latency)
    pub fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms.entry(name.to_string()).or_default().push(value);
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.lock().unwrap().get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.lock().unwrap().get(name).copied()
    }

    /// Get histogram statistics
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.lock().unwrap();
        let values = histograms.get(name)?;
        if values.is_empty() {
            return None;
        }
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = sorted[count / 2];
        let p99 = sorted[(count as f64 * 0.99) as usize];
        Some(HistogramStats { count, sum, mean, p50, p99 })
    }
}

/// Histogram statistics
#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub p50: f64,
    pub p99: f64,
}

/// Telemetry snapshot for export
#[derive(Debug, Clone, serde::Serialize)]
pub struct TelemetrySnapshot {
    pub service: String,
    pub span_count: usize,
    pub counter_count: usize,
    pub gauge_count: usize,
    pub histogram_count: usize,
    pub uptime_secs: f64,
}

impl SigmaTelemetry {
    /// Create a new telemetry instance
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            metrics: MetricsCollector::new(),
            active_spans: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Start a new span for tracing
    pub fn start_span(&self, name: &str, operation: SpanOperation) -> SpanGuard {
        let record = SpanRecord {
            name: name.to_string(),
            service: self.config.service_name.clone(),
            operation,
            start_time: std::time::SystemTime::now(),
            duration: None,
            attributes: Vec::new(),
            status: SpanStatus::Unset,
        };
        SpanGuard {
            record,
            start: Instant::now(),
            telemetry: self,
        }
    }

    /// Get the metrics collector
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics
    }

    /// Record a completed span
    fn record_span(&self, span: SpanRecord) {
        self.metrics.increment("spans.total");
        if matches!(span.status, SpanStatus::Error(_)) {
            self.metrics.increment("spans.errors");
        }
        if let Some(duration) = span.duration {
            let key = format!("span.{}.duration_ms", span.operation);
            self.metrics.record_histogram(&key, duration.as_secs_f64() * 1000.0);
        }
        let mut spans = self.active_spans.lock().unwrap();
        spans.push(span);
    }

    /// Get telemetry snapshot
    pub fn snapshot(&self) -> TelemetrySnapshot {
        let spans = self.active_spans.lock().unwrap();
        let counters = self.metrics.counters.lock().unwrap();
        let gauges = self.metrics.gauges.lock().unwrap();
        let histograms = self.metrics.histograms.lock().unwrap();
        TelemetrySnapshot {
            service: self.config.service_name.clone(),
            span_count: spans.len(),
            counter_count: counters.len(),
            gauge_count: gauges.len(),
            histogram_count: histograms.len(),
            uptime_secs: 0.0,
        }
    }
}

/// RAII span guard that records timing on drop
pub struct SpanGuard<'a> {
    record: SpanRecord,
    start: Instant,
    telemetry: &'a SigmaTelemetry,
}

impl<'a> SpanGuard<'a> {
    /// Add an attribute to the span
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.record.attributes.push((key.to_string(), value.to_string()));
    }

    /// Mark span as OK
    pub fn set_ok(mut self) {
        self.record.status = SpanStatus::Ok;
        self.finish();
    }

    /// Mark span as error
    pub fn set_error(mut self, msg: &str) {
        self.record.status = SpanStatus::Error(msg.to_string());
        self.finish();
    }

    fn finish(mut self) {
        self.record.duration = Some(self.start.elapsed());
        self.telemetry.record_span(self.record.clone());
        std::mem::forget(self); // prevent double record on drop
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        if self.record.duration.is_none() {
            self.record.duration = Some(self.start.elapsed());
            self.record.status = SpanStatus::Ok;
            self.telemetry.record_span(self.record.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_telemetry() -> SigmaTelemetry {
        SigmaTelemetry::new(TelemetryConfig::default())
    }

    #[test]
    fn test_new_telemetry() {
        let t = test_telemetry();
        assert_eq!(t.config.service_name, "ryzanstein");
    }

    #[test]
    fn test_span_lifecycle() {
        let t = test_telemetry();
        let mut span = t.start_span("test_op", SpanOperation::Inference);
        span.set_attribute("model", "bitnet");
        span.set_ok();

        let snap = t.snapshot();
        assert_eq!(snap.span_count, 1);
    }

    #[test]
    fn test_span_auto_close() {
        let t = test_telemetry();
        {
            let _span = t.start_span("auto", SpanOperation::ModelLoad);
        }
        let snap = t.snapshot();
        assert_eq!(snap.span_count, 1);
    }

    #[test]
    fn test_span_error() {
        let t = test_telemetry();
        let span = t.start_span("fail", SpanOperation::TokenGeneration);
        span.set_error("decode failure");

        assert_eq!(t.metrics().get_counter("spans.errors"), 1);
    }

    #[test]
    fn test_metrics_counter() {
        let t = test_telemetry();
        t.metrics().increment("requests");
        t.metrics().increment("requests");
        t.metrics().increment_by("tokens", 42);
        assert_eq!(t.metrics().get_counter("requests"), 2);
        assert_eq!(t.metrics().get_counter("tokens"), 42);
    }

    #[test]
    fn test_metrics_gauge() {
        let t = test_telemetry();
        t.metrics().set_gauge("gpu_utilization", 85.5);
        assert_eq!(t.metrics().get_gauge("gpu_utilization"), Some(85.5));
        assert_eq!(t.metrics().get_gauge("nonexistent"), None);
    }

    #[test]
    fn test_metrics_histogram() {
        let t = test_telemetry();
        for v in [10.0, 20.0, 30.0, 40.0, 50.0] {
            t.metrics().record_histogram("latency", v);
        }
        let stats = t.metrics().get_histogram_stats("latency").unwrap();
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_span_operation_display() {
        assert_eq!(SpanOperation::Inference.to_string(), "inference");
        assert_eq!(SpanOperation::ModelLoad.to_string(), "model.load");
        assert_eq!(SpanOperation::Custom("foo".into()).to_string(), "custom.foo");
    }

    #[test]
    fn test_snapshot() {
        let t = test_telemetry();
        t.start_span("a", SpanOperation::Inference).set_ok();
        t.start_span("b", SpanOperation::ModelLoad).set_ok();
        t.metrics().increment("reqs");
        t.metrics().set_gauge("mem", 42.0);

        let snap = t.snapshot();
        assert_eq!(snap.span_count, 2);
        assert!(snap.counter_count >= 1);
        assert_eq!(snap.gauge_count, 1);
    }
}
