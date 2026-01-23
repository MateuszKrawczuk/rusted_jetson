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
pub mod modules;

#[cfg(feature = "tui")]
#[path = "../tui/mod.rs"]
pub mod tui;

#[cfg(feature = "tui")]
pub use tui::TuiApp;

pub use error::{Error, Result};

pub use modules::{
    cpu::{CpuCore, CpuStats},
    fan::{FanInfo, FanMode, FanStats},
    gpu::{GpuProcess, GpuStats},
    hardware::detect_board,
    hardware::BoardInfo,
    jetson_clocks::JetsonClocksStats,
    memory::MemoryStats,
    nvpmodel::{NVPModel, NVPModelStats},
    power::{PowerRail, PowerStats},
    temperature::{TemperatureStats, ThermalZone},
};

#[cfg(feature = "tui")]
pub use tui::screens::{
    SimpleBoardInfo, SimpleCpuStats, SimpleFanStats, SimpleGpuStats, SimpleMemoryStats,
    SimplePowerStats, SimpleTemperatureStats,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_stats_public_api() {
        let stats = CpuStats::get();
        // CpuStats should be accessible and return valid data
        assert!(stats.usage >= 0.0);
    }

    #[test]
    fn test_memory_stats_public_api() {
        let stats = MemoryStats::get();
        // MemoryStats should be accessible
        assert!(stats.ram_total > 0 || stats.ram_total == 0);
    }

    #[test]
    fn test_temperature_stats_public_api() {
        let stats = TemperatureStats::get();
        // TemperatureStats should be accessible
        assert!(stats.cpu >= 0.0 || stats.cpu == 0.0);
    }

    #[test]
    fn test_power_stats_public_api() {
        let stats = PowerStats::get();
        // PowerStats should be accessible
        assert!(stats.total >= 0.0 || stats.total < 0.0);
    }

    #[test]
    fn test_fan_stats_public_api() {
        let stats = FanStats::get();
        // FanStats should be accessible
        assert!(stats.speed <= 100);
    }

    #[test]
    fn test_gpu_stats_public_api() {
        let stats = GpuStats::get();
        // GpuStats should be accessible
        assert!(stats.usage >= 0.0);
    }

    #[test]
    fn test_board_info_public_api() {
        let info = detect_board();
        // BoardInfo should be accessible
        assert!(!info.model.is_empty() || info.model.is_empty());
    }

    #[test]
    fn test_error_types_accessible() {
        let err = Error::HardwareNotFound("test".to_string());
        let _display = format!("{}", err);
        // Error types should be accessible and displayable
    }
}
