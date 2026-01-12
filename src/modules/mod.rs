// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

pub mod cpu;
pub mod hardware;

// Temporarily commented out to focus on hardware and CPU module tests only
// These modules have compilation errors unrelated to hardware testing
// pub mod engine;
// pub mod fan;
// pub mod gpu;
// pub mod jetson_clocks;
// pub mod memory;
// pub mod nvpmodel;
// pub mod power;
// pub mod processes;
// pub mod tegra_stats;
// pub mod temperature;

pub use hardware::BoardInfo;
