// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

pub mod cpu;
pub mod engine;
pub mod fan;
pub mod gpu;
pub mod hardware;
pub mod jetson_clocks;
pub mod memory;
pub mod nvpmodel;
pub mod power;
pub mod processes;

// Temporarily commented out - need to fix compilation errors
// pub mod tegra_stats;
pub mod temperature;

pub use hardware::BoardInfo;

#[cfg(feature = "tui")]
pub use cpu::{CpuCore, CpuStats};
#[cfg(feature = "tui")]
pub use fan::FanStats;
#[cfg(feature = "tui")]
pub use gpu::GpuStats;
#[cfg(feature = "tui")]
pub use memory::MemoryStats;
#[cfg(feature = "tui")]
pub use power::{PowerRail, PowerStats};
#[cfg(feature = "tui")]
pub use temperature::{TemperatureStats, ThermalZone};
