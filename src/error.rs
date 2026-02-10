use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Span error: {0}")]
    SpanError(String),

    #[error("Metric error: {0}")]
    MetricError(String),

    #[error("Ryzanstein connection error: {0}")]
    RyzansteinError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
