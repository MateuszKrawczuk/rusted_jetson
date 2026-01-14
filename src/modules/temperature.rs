// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Temperature monitoring module

use std::fs;
use std::path::Path;

/// Temperature statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TemperatureStats {
    pub cpu: f32,
    pub gpu: f32,
    pub board: f32,
    pub pmic: f32,
    pub thermal_zones: Vec<ThermalZone>,
}

/// Individual thermal zone
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThermalZone {
    pub index: usize,
    pub name: String,
    pub current_temp: f32,
    pub max_temp: f32,
    pub critical_temp: f32,
}

impl TemperatureStats {
    /// Get current temperature statistics
    pub fn get() -> Self {
        let path = Path::new("/sys/class/thermal");

        if !path.exists() {
            return TemperatureStats::default();
        }

        let mut stats = TemperatureStats::default();
        stats.thermal_zones = read_thermal_zones(path);

        // Extract common temperatures
        for zone in &stats.thermal_zones {
            if zone.name.contains("CPU") || zone.name == "CPU-therm" {
                stats.cpu = zone.current_temp;
            } else if zone.name.contains("GPU") || zone.name == "GPU-therm" {
                stats.gpu = zone.current_temp;
            } else if zone.name.contains("PMIC") {
                stats.pmic = zone.current_temp;
            } else if zone.name.contains("Board") || zone.name.contains("Tboard") {
                stats.board = zone.current_temp;
            }
        }

        stats
    }
}

/// Read all thermal zones
fn read_thermal_zones(base_path: &Path) -> Vec<ThermalZone> {
    let mut zones = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let zone_path = entry.path();
            let zone_name = zone_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Skip if not a thermal zone
            if !zone_name.starts_with("thermal_zone") {
                continue;
            }

            // Parse zone index
            let index = zone_name
                .strip_prefix("thermal_zone")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let type_path = zone_path.join("type");
            let zone_type = fs::read_to_string(type_path)
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Read temperature (in millidegrees)
            let temp_path = zone_path.join("temp");
            let current_temp = fs::read_to_string(temp_path)
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|milli| milli as f32 / 1000.0)
                .unwrap_or(0.0);

            // Read trip point temperatures
            let trip_path = zone_path.join("trip_point_0_temp");
            let max_temp = fs::read_to_string(trip_path)
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|milli| milli as f32 / 1000.0)
                .unwrap_or(0.0);

            // Read critical temperature
            let crit_path = zone_path.join("crit_temp");
            let critical_temp = fs::read_to_string(crit_path)
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|milli| milli as f32 / 1000.0)
                .unwrap_or(0.0);

            zones.push(ThermalZone {
                index,
                name: zone_type,
                current_temp,
                max_temp,
                critical_temp,
            });
        }
    }

    zones
}
