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
