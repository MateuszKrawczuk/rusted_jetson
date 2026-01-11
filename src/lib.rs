// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! # rusted-jetsons Library
//!
//! Fast Rust-based monitoring and control for NVIDIA Jetson devices.
//!
//! ## Features
//!
//! - Hardware detection and monitoring
//! - CPU, GPU, Memory, Power, Temperature monitoring
//! - Control of NVP model, jetson_clocks, fan
//! - OpenTelemetry exports
//! - TUI with ratatui
//!
//! ## License
//!
//! LGPL-3.0 - see LICENSE file for details
//!
//! ## Original Project
//!
//! Forked from jetson-stats by Raffaello Bonghi (AGPL-3.0)
//! <https://github.com/rbonghi/jetson_stats>

pub mod error;
pub mod telemetry;

#[cfg(feature = "telemetry")]
pub use telemetry::TelemetryExporter;

pub use error::{Error, Result};

/// Main Jetson monitor structure
pub struct JetsonMonitor {
    interval: std::time::Duration,
}

impl JetsonMonitor {
    /// Create a new Jetson monitor
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            interval: std::time::Duration::from_secs(1),
        })
    }

    /// Start monitoring
    pub async fn start(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// Jetson statistics structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct JetsonStats {
    pub cpu: CpuStats,
    pub gpu: GpuStats,
    pub memory: MemoryStats,
    pub fan: FanStats,
    pub temperature: TemperatureStats,
    pub power: PowerStats,
    pub board: BoardInfo,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuStats {
    pub usage: f32,
    pub frequency: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuStats {
    pub usage: f32,
    pub frequency: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryStats {
    pub ram_used: u64,
    pub ram_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FanStats {
    pub speed: u8,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemperatureStats {
    pub cpu: f32,
    pub gpu: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PowerStats {
    pub total: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BoardInfo {
    pub model: String,
    pub jetpack: String,
    pub l4t: String,
}
