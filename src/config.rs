use serde::{Deserialize, Serialize};

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Service name for span attribution
    pub service_name: String,
    /// OTLP endpoint for span export
    pub otlp_endpoint: String,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable trace export
    pub traces_enabled: bool,
    /// Ryzanstein API URL
    pub ryzanstein_url: String,
    /// Batch export interval in seconds
    pub export_interval_secs: u64,
    /// Maximum spans to buffer before flush
    pub max_buffer_size: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "ryzanstein".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_rate: 1.0,
            metrics_enabled: true,
            traces_enabled: true,
            ryzanstein_url: "http://localhost:8000".to_string(),
            export_interval_secs: 10,
            max_buffer_size: 1024,
        }
    }
}
