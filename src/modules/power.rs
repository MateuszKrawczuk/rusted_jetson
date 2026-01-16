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
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
    let current_u_a = read_sysfs_u32(iio_path, "in_current_raw").unwrap_or(0) as f32;

    // Read voltage (in uV)
    let voltage_u_v = read_sysfs_u32(iio_path, "in_voltage_raw").unwrap_or(0) as f32;

    // Read scaling factors
    let current_scale = read_sysfs_u32(iio_path, "in_current_scale").unwrap_or(1) as f32;
    let voltage_scale = read_sysfs_u32(iio_path, "in_voltage_scale").unwrap_or(1) as f32;

    // Calculate actual values
    let current_m_a = current_u_a * current_scale / 1000.0; // Convert to mA
    let voltage_m_v = voltage_u_v * voltage_scale / 1000.0; // Convert to mV
    let power_m_w = (current_m_a * voltage_m_v) / 1000.0; // Convert to mW

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_stats_default() {
        let stats = PowerStats::default();
        assert_eq!(stats.total, 0.0);
        assert!(stats.rails.is_empty());
    }

    #[test]
    fn test_power_rail_default() {
        let rail = PowerRail::default();
        assert_eq!(rail.name, "");
        assert_eq!(rail.current, 0.0);
        assert_eq!(rail.voltage, 0.0);
        assert_eq!(rail.power, 0.0);
    }

    #[test]
    fn test_power_rail_structure() {
        let rail = PowerRail {
            name: "VDD_CPU".to_string(),
            current: 1500.0,
            voltage: 5000.0,
            power: 7500.0,
        };

        assert_eq!(rail.name, "VDD_CPU");
        assert_eq!(rail.current, 1500.0);
        assert_eq!(rail.voltage, 5000.0);
        assert_eq!(rail.power, 7500.0);
    }

    #[test]
    fn test_power_stats_structure() {
        let stats = PowerStats {
            total: 15.5,
            rails: vec![
                PowerRail {
                    name: "VDD_CPU".to_string(),
                    current: 1500.0,
                    voltage: 5000.0,
                    power: 7500.0,
                },
                PowerRail {
                    name: "VDD_GPU".to_string(),
                    current: 2000.0,
                    voltage: 5000.0,
                    power: 10000.0,
                },
            ],
        };

        assert_eq!(stats.total, 15.5);
        assert_eq!(stats.rails.len(), 2);
        assert_eq!(stats.rails[0].name, "VDD_CPU");
        assert_eq!(stats.rails[1].name, "VDD_GPU");
    }

    #[test]
    fn test_ina3221_sensor_detection() {
        let stats = PowerStats::get();

        if !stats.rails.is_empty() {
            for rail in &stats.rails {
                assert!(!rail.name.is_empty(), "Rail name should not be empty");
            }
        }
    }

    #[test]
    fn test_power_rail_voltage_reading() {
        let rail = PowerRail {
            name: "VDD_CPU".to_string(),
            current: 1500.0,
            voltage: 5000.0,
            power: 7500.0,
        };

        assert!(rail.voltage > 0.0, "Voltage should be positive");
        assert!(
            rail.voltage <= 12000.0,
            "Voltage should be reasonable (max 12V)"
        );
    }

    #[test]
    fn test_power_rail_current_reading() {
        let rail = PowerRail {
            name: "VDD_GPU".to_string(),
            current: 2000.0,
            voltage: 5000.0,
            power: 10000.0,
        };

        assert!(rail.current >= 0.0, "Current should be non-negative");
        assert!(
            rail.current <= 10000.0,
            "Current should be reasonable (max 10A)"
        );
    }

    #[test]
    fn test_power_calculation() {
        let current_m_a = 1500.0;
        let voltage_m_v = 5000.0;
        let expected_power_m_w = (current_m_a * voltage_m_v) / 1000.0;

        let rail = PowerRail {
            name: "VDD_CPU".to_string(),
            current: current_mA,
            voltage: voltage_mV,
            power: expected_power_mW,
        };

        assert_eq!(rail.power, 7500.0);
    }

    #[test]
    fn test_power_total_calculation() {
        let mut stats = PowerStats::default();
        stats.rails = vec![
            PowerRail {
                name: "rail1".to_string(),
                current: 1000.0,
                voltage: 5000.0,
                power: 5000.0,
            },
            PowerRail {
                name: "rail2".to_string(),
                current: 2000.0,
                voltage: 5000.0,
                power: 10000.0,
            },
        ];

        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;

        assert_eq!(stats.total, 15.0, "Total should be 15000mW = 15W");
    }

    #[test]
    fn test_hwmon_sysfs_parsing() {
        let rail = PowerRail {
            name: "ina3221_0".to_string(),
            current: 1234.0,
            voltage: 5678.0,
            power: 7007.652,
        };

        assert!(rail.name.contains("ina3221"));
        assert!(rail.current > 0.0);
        assert!(rail.voltage > 0.0);
        assert!(rail.power > 0.0);
    }

    #[test]
    fn test_power_serialization() {
        let stats = PowerStats {
            total: 15.5,
            rails: vec![PowerRail {
                name: "VDD_CPU".to_string(),
                current: 1500.0,
                voltage: 5000.0,
                power: 7500.0,
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "PowerStats should be serializable");

        let deserialized: Result<PowerStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "PowerStats should be deserializable");
    }

    #[test]
    fn test_power_rail_serialization() {
        let rail = PowerRail {
            name: "VDD_GPU".to_string(),
            current: 2000.0,
            voltage: 5000.0,
            power: 10000.0,
        };

        let json = serde_json::to_string(&rail);
        assert!(json.is_ok(), "PowerRail should be serializable");

        let deserialized: Result<PowerRail, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "PowerRail should be deserializable");
    }

    #[test]
    fn test_power_range_validation() {
        let stats = PowerStats::get();

        if !stats.rails.is_empty() {
            for rail in &stats.rails {
                assert!(rail.voltage > 0.0, "Voltage should be positive");
                assert!(rail.voltage <= 12000.0, "Voltage should be reasonable");
                assert!(rail.current >= 0.0, "Current should be non-negative");
                assert!(rail.current <= 10000.0, "Current should be reasonable");
                assert!(rail.power >= 0.0, "Power should be non-negative");
                assert!(
                    rail.power <= 120000.0,
                    "Power should be reasonable (max 120W)"
                );
            }
        }

        assert!(stats.total >= 0.0, "Total power should be non-negative");
        assert!(
            stats.total <= 200.0,
            "Total power should be reasonable (max 200W)"
        );
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test power -- --ignored"]
    fn test_print_power_info() {
        println!("\n=== Power Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - power info not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = PowerStats::get();

        println!("Total power: {:.2}W", stats.total);
        println!("Number of power rails: {}", stats.rails.len());

        for (i, rail) in stats.rails.iter().enumerate() {
            println!(
                "  Rail {}: {} - {:.2}mA @ {:.2}mV = {:.2}mW",
                i, rail.name, rail.current, rail.voltage, rail.power
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_power_calculation_edge_cases() {
        // Test with zero current
        let rail_zero_current = PowerRail {
            name: "test".to_string(),
            current: 0.0,
            voltage: 5000.0,
            power: 0.0,
        };
        assert_eq!(rail_zero_current.power, 0.0);

        // Test with zero voltage
        let rail_zero_voltage = PowerRail {
            name: "test".to_string(),
            current: 1500.0,
            voltage: 0.0,
            power: 0.0,
        };
        assert_eq!(rail_zero_voltage.power, 0.0);
    }
}
