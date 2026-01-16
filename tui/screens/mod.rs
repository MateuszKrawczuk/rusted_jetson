// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI screens module

pub mod all;
pub mod control;
pub mod cpu_screen;
pub mod gpu;
pub mod info;
pub mod memory;
pub mod power;
pub mod temperature;

pub use all::AllScreen;
pub use control::{ControlScreen, ControlStats};
pub use cpu::CpuScreen;
pub use cpu_screen as cpu;
pub use gpu::GpuScreen;
pub use info::{InfoScreen, InfoStats};

pub use memory::MemoryScreen;
pub use power::PowerScreen;
pub use temperature::TemperatureScreen;

// Re-export Simple*Stats and ScreenStats from individual modules
pub use cpu_screen::{CoreStats, CpuScreenStats, SimpleCpuStats, SimpleFanStats};
pub use gpu::{GpuScreenStats, SimpleGpuStats};
pub use info::SimpleBoardInfo;
pub use memory::{MemoryScreenStats, SimpleMemoryStats};
pub use power::{PowerRail, PowerScreenStats, SimplePowerStats};
pub use temperature::{SimpleTemperatureStats, TemperatureScreenStats, ThermalZone};

#[derive(Debug, Clone, serde::Serialize)]
pub struct JetsonStats {
    pub cpu: SimpleCpuStats,
    pub gpu: SimpleGpuStats,
    pub memory: SimpleMemoryStats,
    pub fan: SimpleFanStats,
    pub temperature: SimpleTemperatureStats,
    pub power: SimplePowerStats,
    pub board: SimpleBoardInfo,
}
