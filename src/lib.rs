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
pub mod modules;

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "telemetry")]
pub use telemetry::TelemetryExporter;

pub use error::{Error, Result};

pub use modules::{
    cpu::{CpuStats, get_core_count},
    gpu::GpuStats,
    memory::MemoryStats,
    fan::FanStats,
    temperature::{TemperatureStats, ThermalZone},
    power::{PowerStats, PowerRail},
    hardware::BoardInfo,
};

/// Re-export simple types for TUI compatibility
pub use CpuStats as SimpleCpuStats;
pub use GpuStats as SimpleGpuStats;
pub use MemoryStats as SimpleMemoryStats;
pub use FanStats as SimpleFanStats;
pub use TemperatureStats as SimpleTemperatureStats;
pub use PowerStats as SimplePowerStats;
pub use BoardInfo as SimpleBoardInfo;

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

/// Simple CPU stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleCpuStats {
    pub usage: f32,
    pub frequency: u32,
}

/// Simple GPU stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleGpuStats {
    pub usage: f32,
    pub frequency: u32,
}

/// Simple memory stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleMemoryStats {
    pub ram_used: u64,
    pub ram_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

/// Simple fan stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleFanStats {
    pub speed: u8,
}

/// Simple temperature stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleTemperatureStats {
    pub cpu: f32,
    pub gpu: f32,
}

/// Simple power stats for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimplePowerStats {
    pub total: f32,
}

/// Simple board info for TUI
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleBoardInfo {
    pub model: String,
    pub jetpack: String,
    pub l4t: String,
}
