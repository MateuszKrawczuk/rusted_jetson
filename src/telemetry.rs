// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! OpenTelemetry exports for rusted-jetsons

#[cfg(feature = "telemetry")]
pub struct TelemetryExporter {
    endpoint: String,
}

#[cfg(feature = "telemetry")]
impl TelemetryExporter {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn export(&self, stats: &crate::JetsonStats) -> anyhow::Result<()> {
        // TODO: Implement OTLP export
        Ok(())
    }
}

#[cfg(all(test, feature = "telemetry"))]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_exporter_new() {
        let exporter = TelemetryExporter::new("http://localhost:4318".to_string());
        assert_eq!(exporter.endpoint(), "http://localhost:4318");
    }

    #[test]
    fn test_telemetry_exporter_endpoint() {
        let exporter = TelemetryExporter::new("http://grafana:4318/v1/metrics".to_string());
        assert!(exporter.endpoint().starts_with("http://"));
        assert!(exporter.endpoint().contains("4318"));
    }
}
