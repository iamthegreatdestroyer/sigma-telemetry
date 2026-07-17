//! Python (pyo3) bindings — feature `python`. Lets the Ryzanstein gateway (Python)
//! use this crate's real MetricsCollector + Prometheus text exposition in-process.
#![cfg(feature = "python")]
use pyo3::prelude::*;
use std::sync::Arc;

use crate::prometheus::format_metrics;
use crate::MetricsCollector;

/// Thin Python handle over a real Rust MetricsCollector.
#[pyclass]
pub struct PyMetrics {
    inner: Arc<MetricsCollector>,
}

#[pymethods]
impl PyMetrics {
    #[new]
    fn new() -> Self {
        PyMetrics { inner: Arc::new(MetricsCollector::new()) }
    }
    fn increment(&self, name: &str) { self.inner.increment(name); }
    fn increment_by(&self, name: &str, value: u64) { self.inner.increment_by(name, value); }
    fn record_histogram(&self, name: &str, value: f64) { self.inner.record_histogram(name, value); }
    fn set_gauge(&self, name: &str, value: f64) { self.inner.set_gauge(name, value); }
    fn get_counter(&self, name: &str) -> u64 { self.inner.get_counter(name) }
    /// Prometheus text exposition of all metrics (histograms emit _count/_sum/_p50/_p95/_p99).
    fn render(&self) -> String { format_metrics(&self.inner) }
}

#[pymodule]
fn sigma_telemetry(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMetrics>()?;
    Ok(())
}
