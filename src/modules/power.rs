// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Power monitoring module

use std::fs;
use std::path::Path;

/// Power statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PowerStats {
    pub total: f32,
    pub rails: Vec<PowerRail>,
}

/// Individual power rail
#[derive(Debug, Clone, Default)]
pub struct PowerRail {
    pub name: String,
    pub current: f32, // mA
    pub voltage: f32, // mV
    pub power: f32,   // mW
}

impl PowerStats {
    /// Get current power statistics
    pub fn get() -> Self {
        let path = Path::new("/sys/bus/i2c/devices");

        if !path.exists() {
            return PowerStats::default();
        }

        let mut stats = PowerStats::default();
        stats.rails = read_power_rails(path);

        // Calculate total power
        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0; // Convert mW to W

        stats
    }
}

/// Read all power rails from I2C devices
fn read_power_rails(base_path: &Path) -> Vec<PowerRail> {
    let mut rails = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let i2c_path = entry.path();

            // Look for iio:device directories
            if !i2c_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("iio:device"))
                .unwrap_or(false)
            {
                continue;
            }

            // Try to read INA3221 device
            if let Some(rail) = read_ina3221_rail(&i2c_path) {
                rails.push(rail);
            }
        }
    }

    rails
}

/// Read INA3221 power rail
fn read_ina3221_rail(iio_path: &Path) -> Option<PowerRail> {
    let name_path = iio_path.join("name");

    // Get rail name
    let name = fs::read_to_string(name_path)
        .ok()
        .map(|s| s.trim().to_string())?;

    // Read current (in uA)
    let current_uA = read_sysfs_u32(iio_path, "in_current_raw").unwrap_or(0) as f32;

    // Read voltage (in uV)
    let voltage_uV = read_sysfs_u32(iio_path, "in_voltage_raw").unwrap_or(0) as f32;

    // Read scaling factors
    let current_scale = read_sysfs_u32(iio_path, "in_current_scale").unwrap_or(1) as f32;
    let voltage_scale = read_sysfs_u32(iio_path, "in_voltage_scale").unwrap_or(1) as f32;

    // Calculate actual values
    let current_mA = current_uA * current_scale / 1000.0; // Convert to mA
    let voltage_mV = voltage_uV * voltage_scale / 1000.0; // Convert to mV
    let power_mW = (current_mA * voltage_mV) / 1000.0; // Convert to mW

    Some(PowerRail {
        name,
        current: current_mA,
        voltage: voltage_mV,
        power: power_mW,
    })
}

/// Read a u32 value from sysfs
fn read_sysfs_u32(path: &Path, file: &str) -> Option<u32> {
    let file_path = path.join(file);

    fs::read_to_string(file_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
