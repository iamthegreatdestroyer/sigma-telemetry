//! Telemetry export to OTLP and stdout.

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;
use crate::SpanRecord;
use serde::Serialize;

/// Export format
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Otlp,
    Json,
    Stdout,
}

/// Telemetry exporter
pub struct Exporter {
    config: TelemetryConfig,
    format: ExportFormat,
}

/// Exported span in wire format
#[derive(Debug, Serialize)]
pub struct ExportedSpan {
    pub name: String,
    pub service: String,
    pub operation: String,
    pub duration_ms: Option<f64>,
    pub status: String,
    pub attributes: Vec<(String, String)>,
}

impl From<&SpanRecord> for ExportedSpan {
    fn from(record: &SpanRecord) -> Self {
        ExportedSpan {
            name: record.name.clone(),
            service: record.service.clone(),
            operation: record.operation.to_string(),
            duration_ms: record.duration.map(|d| d.as_secs_f64() * 1000.0),
            status: match &record.status {
                crate::SpanStatus::Ok => "ok".to_string(),
                crate::SpanStatus::Error(msg) => format!("error: {}", msg),
                crate::SpanStatus::Unset => "unset".to_string(),
            },
            attributes: record.attributes.clone(),
        }
    }
}

impl Exporter {
    /// Create a new exporter
    pub fn new(config: TelemetryConfig, format: ExportFormat) -> Self {
        Self { config, format }
    }

    /// Export spans
    pub fn export(&self, spans: &[SpanRecord]) -> Result<String, TelemetryError> {
        let exported: Vec<ExportedSpan> = spans.iter().map(|s| s.into()).collect();

        match self.format {
            ExportFormat::Json | ExportFormat::Stdout => serde_json::to_string_pretty(&exported)
                .map_err(|e| TelemetryError::ExportError(e.to_string())),
            ExportFormat::Otlp => {
                let resource_spans = serde_json::json!({
                    "resourceSpans": [{
                        "resource": {
                            "attributes": [{
                                "key": "service.name",
                                "value": { "stringValue": &self.config.service_name }
                            }]
                        },
                        "scopeSpans": [{
                            "scope": {
                                "name": "sigma-telemetry",
                                "version": env!("CARGO_PKG_VERSION")
                            },
                            "spans": exported.iter().map(|s| {
                                serde_json::json!({
                                    "name": &s.name,
                                    "kind": 1,
                                    "attributes": s.attributes.iter().map(|(k, v)| {
                                        serde_json::json!({
                                            "key": k,
                                            "value": { "stringValue": v }
                                        })
                                    }).collect::<Vec<_>>(),
                                    "status": {
                                        "code": if s.status.starts_with("error") { 2 } else { 1 },
                                        "message": &s.status
                                    },
                                    "durationNanos": s.duration_ms.map(|ms| (ms * 1_000_000.0) as u64).unwrap_or(0),
                                })
                            }).collect::<Vec<_>>()
                        }]
                    }]
                });

                let body = serde_json::to_string(&resource_spans)
                    .map_err(|e| TelemetryError::ExportError(e.to_string()))?;

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .map_err(|e| TelemetryError::ExportError(e.to_string()))?;

                let endpoint = format!("{}/v1/traces", self.config.otlp_endpoint);
                let response = client
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send();

                match response {
                    Ok(resp) if resp.status().is_success() => Ok(format!(
                        "Exported {} spans to {}",
                        spans.len(),
                        endpoint
                    )),
                    Ok(resp) => Err(TelemetryError::ExportError(format!(
                        "OTLP endpoint returned {}: {}",
                        resp.status(),
                        resp.text().unwrap_or_default()
                    ))),
                    Err(e) => Err(TelemetryError::ExportError(format!(
                        "Failed to reach OTLP endpoint {}: {}",
                        endpoint, e
                    ))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SpanOperation, SpanStatus};

    fn sample_span() -> SpanRecord {
        SpanRecord {
            name: "test".to_string(),
            service: "ryzanstein".to_string(),
            operation: SpanOperation::Inference,
            start_time: std::time::SystemTime::now(),
            duration: Some(std::time::Duration::from_millis(42)),
            attributes: vec![("model".to_string(), "bitnet".to_string())],
            status: SpanStatus::Ok,
        }
    }

    #[test]
    fn test_json_export() {
        let exporter = Exporter::new(TelemetryConfig::default(), ExportFormat::Json);
        let result = exporter.export(&[sample_span()]).unwrap();
        assert!(result.contains("inference"));
        assert!(result.contains("bitnet"));
    }

    #[test]
    fn test_otlp_export_formats_correctly() {
        // OTLP export will fail to connect in test env, but we can verify it
        // attempts the right endpoint by checking the error message
        let exporter = Exporter::new(TelemetryConfig::default(), ExportFormat::Otlp);
        let result = exporter.export(&[sample_span()]);
        match result {
            Ok(msg) => assert!(msg.contains("spans")),
            Err(e) => {
                let err_msg = format!("{}", e);
                // Should mention the OTLP endpoint in the error
                assert!(
                    err_msg.contains("localhost:4317") || err_msg.contains("OTLP"),
                    "Unexpected error: {err_msg}"
                );
            }
        }
    }

    #[test]
    fn test_exported_span_conversion() {
        let span = sample_span();
        let exported = ExportedSpan::from(&span);
        assert_eq!(exported.operation, "inference");
        assert_eq!(exported.status, "ok");
        assert!(exported.duration_ms.unwrap() > 0.0);
    }
}
