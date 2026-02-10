//! Ryzanstein-specific telemetry integration.

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;

/// Client for Ryzanstein telemetry hooks
pub struct RyzansteinTelemetryClient {
    base_url: String,
    client: Option<reqwest::Client>,
}

/// Health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub model_loaded: bool,
    pub inference_count: u64,
    pub uptime_secs: f64,
}

impl RyzansteinTelemetryClient {
    pub fn new(config: &TelemetryConfig) -> Self {
        Self {
            base_url: config.ryzanstein_url.clone(),
            client: reqwest::Client::builder().build().ok(),
        }
    }

    /// Probe Ryzanstein health
    pub async fn health_check(&self) -> Result<HealthStatus, TelemetryError> {
        let url = format!("{}/health", self.base_url);
        let client = self.client.as_ref()
            .ok_or_else(|| TelemetryError::RyzansteinError("HTTP client not initialized".into()))?;

        let resp = client.get(&url).send().await
            .map_err(|e| TelemetryError::RyzansteinError(e.to_string()))?;

        resp.json::<HealthStatus>().await
            .map_err(|e| TelemetryError::RyzansteinError(e.to_string()))
    }

    /// Push telemetry data to Ryzanstein
    pub async fn push_metrics(&self, snapshot: &crate::TelemetrySnapshot) -> Result<(), TelemetryError> {
        let url = format!("{}/v1/telemetry", self.base_url);
        let client = self.client.as_ref()
            .ok_or_else(|| TelemetryError::RyzansteinError("HTTP client not initialized".into()))?;

        client.post(&url)
            .json(snapshot)
            .send()
            .await
            .map_err(|e| TelemetryError::RyzansteinError(e.to_string()))?;

        Ok(())
    }

    /// Fallback health status when Ryzanstein is unavailable
    pub fn fallback_health() -> HealthStatus {
        HealthStatus {
            status: "unavailable".to_string(),
            model_loaded: false,
            inference_count: 0,
            uptime_secs: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;

    #[test]
    fn test_client_creation() {
        let client = RyzansteinTelemetryClient::new(&TelemetryConfig::default());
        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_fallback_health() {
        let health = RyzansteinTelemetryClient::fallback_health();
        assert_eq!(health.status, "unavailable");
        assert!(!health.model_loaded);
    }

    #[test]
    fn test_health_serialization() {
        let health = HealthStatus {
            status: "healthy".to_string(),
            model_loaded: true,
            inference_count: 42,
            uptime_secs: 3600.0,
        };
        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("healthy"));

        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.inference_count, 42);
    }
}
