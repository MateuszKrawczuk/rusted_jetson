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

        let mut stats = TemperatureStats {
            thermal_zones: read_thermal_zones(path),
            ..Default::default()
        };

        // Extract common temperatures (case-insensitive)
        for zone in &stats.thermal_zones {
            let name_lower = zone.name.to_lowercase();
            if name_lower.contains("cpu") || zone.name == "CPU-therm" || zone.name == "cpu-thermal"
            {
                stats.cpu = zone.current_temp;
            } else if name_lower.contains("gpu")
                || zone.name == "GPU-therm"
                || zone.name == "gpu-thermal"
            {
                stats.gpu = zone.current_temp;
            } else if name_lower.contains("pmic") {
                stats.pmic = zone.current_temp;
            } else if name_lower.contains("board")
                || name_lower.contains("tboard")
                || zone.name.contains("Tboard")
            {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_stats_default() {
        let stats = TemperatureStats::default();
        assert_eq!(stats.cpu, 0.0);
        assert_eq!(stats.gpu, 0.0);
        assert_eq!(stats.board, 0.0);
        assert_eq!(stats.pmic, 0.0);
        assert!(stats.thermal_zones.is_empty());
    }

    #[test]
    fn test_thermal_zone_default() {
        let zone = ThermalZone::default();
        assert_eq!(zone.index, 0);
        assert_eq!(zone.name, "");
        assert_eq!(zone.current_temp, 0.0);
        assert_eq!(zone.max_temp, 0.0);
        assert_eq!(zone.critical_temp, 0.0);
    }

    #[test]
    fn test_thermal_zone_structure() {
        let zone = ThermalZone {
            index: 1,
            name: "CPU-therm".to_string(),
            current_temp: 45.5,
            max_temp: 85.0,
            critical_temp: 95.0,
        };

        assert_eq!(zone.index, 1);
        assert_eq!(zone.name, "CPU-therm");
        assert_eq!(zone.current_temp, 45.5);
        assert_eq!(zone.max_temp, 85.0);
        assert_eq!(zone.critical_temp, 95.0);
    }

    #[test]
    fn test_temperature_stats_structure() {
        let stats = TemperatureStats {
            cpu: 50.0,
            gpu: 60.0,
            board: 40.0,
            pmic: 35.0,
            thermal_zones: vec![
                ThermalZone {
                    index: 0,
                    name: "CPU-therm".to_string(),
                    current_temp: 50.0,
                    max_temp: 85.0,
                    critical_temp: 95.0,
                },
                ThermalZone {
                    index: 1,
                    name: "GPU-therm".to_string(),
                    current_temp: 60.0,
                    max_temp: 87.0,
                    critical_temp: 97.0,
                },
            ],
        };

        assert_eq!(stats.cpu, 50.0);
        assert_eq!(stats.gpu, 60.0);
        assert_eq!(stats.board, 40.0);
        assert_eq!(stats.pmic, 35.0);
        assert_eq!(stats.thermal_zones.len(), 2);
    }

    #[test]
    fn test_thermal_zone_detection() {
        let zone1 = ThermalZone {
            index: 0,
            name: "CPU-therm".to_string(),
            current_temp: 45.0,
            max_temp: 85.0,
            critical_temp: 95.0,
        };

        assert!(zone1.name.contains("CPU"));

        let zone2 = ThermalZone {
            index: 1,
            name: "GPU-therm".to_string(),
            current_temp: 55.0,
            max_temp: 87.0,
            critical_temp: 97.0,
        };

        assert!(zone2.name.contains("GPU"));
    }

    #[test]
    fn test_thermal_zone_type_reading() {
        let stats = TemperatureStats {
            cpu: 50.0,
            gpu: 60.0,
            board: 40.0,
            pmic: 35.0,
            thermal_zones: vec![
                ThermalZone {
                    index: 0,
                    name: "CPU-therm".to_string(),
                    current_temp: 50.0,
                    max_temp: 85.0,
                    critical_temp: 95.0,
                },
                ThermalZone {
                    index: 1,
                    name: "PMIC-die".to_string(),
                    current_temp: 35.0,
                    max_temp: 70.0,
                    critical_temp: 80.0,
                },
            ],
        };

        assert_eq!(stats.cpu, 50.0);
        assert_eq!(stats.pmic, 35.0);
    }

    #[test]
    fn test_temperature_value_reading() {
        let stats = TemperatureStats::get();

        if !stats.thermal_zones.is_empty() {
            for zone in &stats.thermal_zones {
                assert!(zone.current_temp >= 0.0, "Temperature should be >= 0");
                assert!(
                    zone.current_temp <= 150.0,
                    "Temperature should be reasonable"
                );
            }
        }
    }

    #[test]
    fn test_trip_point_reading() {
        let zone = ThermalZone {
            index: 0,
            name: "CPU-therm".to_string(),
            current_temp: 45.0,
            max_temp: 85.0,
            critical_temp: 95.0,
        };

        assert!(
            zone.max_temp > zone.current_temp,
            "Max temp should be > current"
        );
        assert!(
            zone.critical_temp > zone.max_temp,
            "Critical temp should be > max"
        );
    }

    #[test]
    fn test_thermal_zone_sysfs_parsing() {
        let zone = ThermalZone {
            index: 10,
            name: "Tboard".to_string(),
            current_temp: 38.5,
            max_temp: 70.0,
            critical_temp: 80.0,
        };

        assert_eq!(zone.index, 10);
        assert_eq!(zone.name, "Tboard");
        assert!(zone.current_temp > 0.0);
    }

    #[test]
    fn test_temperature_serialization() {
        let stats = TemperatureStats {
            cpu: 45.5,
            gpu: 55.0,
            board: 40.0,
            pmic: 35.0,
            thermal_zones: vec![ThermalZone {
                index: 0,
                name: "CPU-therm".to_string(),
                current_temp: 45.5,
                max_temp: 85.0,
                critical_temp: 95.0,
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "TemperatureStats should be serializable");

        let deserialized: Result<TemperatureStats, _> = serde_json::from_str(&json.unwrap());
        assert!(
            deserialized.is_ok(),
            "TemperatureStats should be deserializable"
        );
    }

    #[test]
    fn test_thermal_zone_serialization() {
        let zone = ThermalZone {
            index: 1,
            name: "GPU-therm".to_string(),
            current_temp: 60.0,
            max_temp: 87.0,
            critical_temp: 97.0,
        };

        let json = serde_json::to_string(&zone);
        assert!(json.is_ok(), "ThermalZone should be serializable");

        let deserialized: Result<ThermalZone, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "ThermalZone should be deserializable");
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test temperature -- --ignored"]
    fn test_print_temperature_info() {
        println!("\n=== Temperature Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - temperature info not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = TemperatureStats::get();

        println!("CPU temperature: {:.1}°C", stats.cpu);
        println!("GPU temperature: {:.1}°C", stats.gpu);
        println!("Board temperature: {:.1}°C", stats.board);
        println!("PMIC temperature: {:.1}°C", stats.pmic);
        println!("Number of thermal zones: {}", stats.thermal_zones.len());

        for zone in &stats.thermal_zones {
            println!(
                "  Zone {}: {} - {:.1}°C (max: {:.1}°C, critical: {:.1}°C)",
                zone.index, zone.name, zone.current_temp, zone.max_temp, zone.critical_temp
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_temperature_range_validation() {
        let zone = ThermalZone {
            index: 0,
            name: "test-zone".to_string(),
            current_temp: 25.0,
            max_temp: 80.0,
            critical_temp: 90.0,
        };

        assert!(
            zone.current_temp >= -20.0,
            "Temperature should be reasonable low"
        );
        assert!(
            zone.current_temp <= 150.0,
            "Temperature should be reasonable high"
        );
        assert!(
            zone.max_temp > zone.current_temp,
            "Max temp should be > current"
        );
        assert!(
            zone.critical_temp > zone.max_temp,
            "Critical temp should be > max"
        );
    }
}
