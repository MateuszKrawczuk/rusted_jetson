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

/// Simple CPU stats for TUI (imported from modules)
pub use cpu::CpuStats;
pub use gpu::GpuStats;
pub use memory::MemoryStats;
pub use fan::FanStats;
pub use temperature::TemperatureStats;
pub use power::PowerStats;
pub use hardware::BoardInfo;

/// Simple temperature stats (without zones for TUI)
pub use temperature::{TemperatureStats as SimpleTemperatureStats};

/// Simple power stats (without rails for TUI)
pub use power::PowerStats as SimplePowerStats;

/// Simple board info (without serial for TUI)
pub use hardware::BoardInfo as SimpleBoardInfo;
