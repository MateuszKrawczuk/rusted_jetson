// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Power monitoring module

use std::fs;
use std::path::Path;

/// Power statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PowerStats {
    pub total: f32,
    pub rails: Vec<PowerRail>,
}

/// Individual power rail
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PowerRail {
    pub name: String,
    pub current: f32,
    pub voltage: f32,
    pub power: f32,
}


impl PowerStats {
    /// Get current power statistics
    pub fn get() -> Self {
        let mut stats = PowerStats::default();

        let i2c_path = Path::new("/sys/bus/i2c/devices");

        if !i2c_path.exists() {
            return PowerStats::default();
        }

        stats.rails = read_power_rails(&i2c_path);

        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;

        stats
    }
}
