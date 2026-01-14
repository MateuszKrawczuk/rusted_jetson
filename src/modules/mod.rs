// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

pub mod cpu;
pub mod gpu;
pub mod hardware;
pub mod memory;

// Temporarily commented out - need to fix compilation errors
// pub mod engine;
// pub mod fan;
// pub mod jetson_clocks;
// pub mod nvpmodel;
// pub mod power;
// pub mod processes;
// pub mod tegra_stats;
// pub mod temperature;

pub use hardware::BoardInfo;
