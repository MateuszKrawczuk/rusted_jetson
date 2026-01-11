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

    pub async fn export(&self, stats: &crate::JetsonStats) -> anyhow::Result<()> {
        // TODO: Implement OTLP export
        Ok(())
    }
}
