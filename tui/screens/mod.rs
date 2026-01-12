// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI screens module

pub mod all;
pub mod control;
pub mod cpu_screen as cpu;
pub mod info;
pub mod gpu;
pub mod memory;
pub mod power;
pub mod temperature;

pub use all::AllScreen;
pub use control::{ControlScreen, ControlStats};
pub use info::{InfoScreen, InfoStats};
pub use cpu::CpuScreen;
pub use gpu::GpuScreen;
pub use memory::MemoryScreen;
pub use power::PowerScreen;
pub use temperature::TemperatureScreen;
