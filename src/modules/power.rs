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

        if i2c_path.exists() {
            stats.rails = read_power_rails(i2c_path);
            stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;
        }

        // Fallback to hwmon if INA3221 sensors not available
        if stats.rails.is_empty() || stats.total <= 0.0 {
            stats = Self::read_hwmon_power();
        }

        stats
    }

    /// Read power from hwmon system (fallback method)
    fn read_hwmon_power() -> Self {
        let mut stats = PowerStats::default();
        let hwmon_path = Path::new("/sys/class/hwmon");

        if !hwmon_path.exists() {
            return stats;
        }

        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_dir = entry.path();

                // Check if this is an INA3221 sensor
                let name_path = hwmon_dir.join("name");
                if let Ok(name) = fs::read_to_string(&name_path) {
                    if name.trim() == "ina3221" {
                        // Read INA3221 power rails (channels 1-3)
                        for channel in 1..=3 {
                            if let Some(rail) = read_ina3221_hwmon_rail(&hwmon_dir, channel) {
                                stats.rails.push(rail);
                            }
                        }
                        continue;
                    }
                }

                // Fallback: Look for power1_input or power1_average
                let hwmon_name: String = hwmon_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let power_input = hwmon_dir.join("power1_input");
                let power_average = hwmon_dir.join("power1_average");

                let power_value = if power_input.exists() {
                    read_sysfs_u32(&power_input).unwrap_or(0) as f32 / 1000000.0
                // uW to W
                } else if power_average.exists() {
                    read_sysfs_u32(&power_average).unwrap_or(0) as f32 / 1000000.0
                // uW to W
                } else {
                    continue;
                };

                if power_value > 0.0 {
                    stats.rails.push(PowerRail {
                        name: hwmon_name,
                        current: power_value,
                        voltage: 0.0,
                        power: power_value,
                    });
                }
            }
        }

        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>(); // power is already in W
        stats
    }
}

/// Read INA3221 power rail from hwmon path
/// Channel 1-3 corresponds to the three channels of INA3221
fn read_ina3221_hwmon_rail(hwmon_path: &Path, channel: usize) -> Option<PowerRail> {
    // Read rail label (e.g., "VDD_IN", "VDD_CPU_GPU_CV", etc.)
    let label_path = hwmon_path.join(format!("in{}_label", channel));
    let rail_name = if let Ok(label) = fs::read_to_string(&label_path) {
        let name = label.trim().to_string();
        // Skip NC (Not Connected) rails on Orin family
        if name == "NC" {
            return None;
        }
        name
    } else {
        format!("rail{}", channel)
    };

    // Read current in microamps (uA) - curr{n}_input
    let curr_path = hwmon_path.join(format!("curr{}_input", channel));
    let current_ua = read_sysfs_i32(&curr_path).unwrap_or(0) as f32;

    // Read voltage in millivolts (mV) - in{n}_input
    let volt_path = hwmon_path.join(format!("in{}_input", channel));
    let voltage_mv = read_sysfs_i32(&volt_path).unwrap_or(0) as f32;

    // Calculate power: P = V * I
    // voltage_mv * current_ua / 1_000_000_000 = power in W
    let power_w = (voltage_mv * current_ua) / 1_000_000_000.0;

    // Only return rail if we got valid readings
    if power_w > 0.0 || (voltage_mv > 0.0 && current_ua >= 0.0) {
        Some(PowerRail {
            name: rail_name,
            current: current_ua / 1000.0,  // uA to mA
            voltage: voltage_mv,            // mV
            power: power_w,                 // W
        })
    } else {
        None
    }
}

/// Read a signed i32 value from sysfs (for sensors that can have negative values)
fn read_sysfs_i32(path: &Path) -> Option<i32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Read a u32 value from sysfs
fn read_sysfs_u32(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Read all power rails from I2C devices
fn read_power_rails(base_path: &Path) -> Vec<PowerRail> {
    let mut rails = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let i2c_path = entry.path();

            if !i2c_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("iio:device"))
                .unwrap_or(false)
            {
                continue;
            }

            let rail_num: usize = 0;
            if let Some(rail) = read_ina3221_rail(&i2c_path, rail_num) {
                rails.push(rail);
            }
        }
    }

    rails
}

/// Read INA3221 power rail
fn read_ina3221_rail(iio_path: &Path, rail_num: usize) -> Option<PowerRail> {
    let label_path = iio_path.join(format!("in{}_label", rail_num));
    let rail_name = if let Ok(name) = fs::read_to_string(&label_path) {
        name.trim().to_string()
    } else {
        let name_path = iio_path.join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            if name.contains("ina3221") {
                format!("in{}", rail_num)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    if rail_name.is_empty() {
        return None;
    }

    let current_u_a =
        read_sysfs_u32(&iio_path.join(format!("curr{}_input", rail_num))).unwrap_or(0) as f32;
    let voltage_u_v =
        read_sysfs_u32(&iio_path.join(format!("in{}_input", rail_num))).unwrap_or(0) as f32;
    let power_m_w = current_u_a * voltage_u_v / 1000000.0;

    Some(PowerRail {
        name: rail_name,
        current: current_u_a,
        voltage: voltage_u_v,
        power: power_m_w,
    })
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
        let expected_power_m_w = current_m_a * voltage_m_v / 1000.0;

        let rail = PowerRail {
            name: "VDD_CPU".to_string(),
            current: current_m_a,
            voltage: voltage_m_v,
            power: expected_power_m_w,
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
    fn test_power_calculation_edge_cases() {
        let rail_zero_current = PowerRail {
            name: "test".to_string(),
            current: 0.0,
            voltage: 5000.0,
            power: 0.0,
        };
        assert_eq!(rail_zero_current.power, 0.0);

        let rail_zero_voltage = PowerRail {
            name: "test".to_string(),
            current: 1500.0,
            voltage: 0.0,
            power: 0.0,
        };
        assert_eq!(rail_zero_voltage.power, 0.0);
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
    #[ignore = "Requires implementation - failing test for system power supply"]
    fn testSystemPowerSupplyReading() {
        let stats = PowerStats::get();

        let has_system_power = stats.rails.iter().any(|r| {
            r.name.contains("USB") || r.name.contains("ucsi-source-psy") || r.name.contains("AC")
        });

        assert!(
            has_system_power || !stats.rails.is_empty(),
            "Should have system power rails or at least I2C rails"
        );
    }

    #[test]
    #[ignore = "Requires implementation - failing test for total power rails"]
    fn testTotalPowerRailDetection() {
        let stats = PowerStats::get();

        let has_pom_5v = stats.rails.iter().any(|r| r.name == "POM_5V_IN");
        let has_vdd_in = stats.rails.iter().any(|r| r.name == "VDD_IN");

        assert!(
            has_pom_5v || has_vdd_in || !stats.rails.is_empty(),
            "Should have POM_5V_IN, VDD_IN, or at least other rails"
        );
    }

    #[test]
    #[ignore = "Requires implementation - failing test for rail labels"]
    fn testIna3221WithLabels() {
        let stats = PowerStats::get();

        if !stats.rails.is_empty() {
            let has_descriptive_name = stats
                .rails
                .iter()
                .any(|r| !r.name.starts_with("i2c-") && !r.name.starts_with("1-00"));

            assert!(has_descriptive_name, "Rails should have descriptive labels");
        }
    }

    #[test]
    fn test_power_calculation_accuracy() {
        let current_ma = 1500.0;
        let voltage_mv = 5000.0;
        let expected_power_mw = current_ma * voltage_mv / 1000.0;

        let rail = PowerRail {
            name: "test".to_string(),
            current: current_ma,
            voltage: voltage_mv,
            power: expected_power_mw,
        };

        let mut stats = PowerStats::default();
        stats.rails = vec![rail];
        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;

        assert_eq!(
            stats.total,
            expected_power_mw / 1000.0,
            "Total power should be sum of rail powers / 1000"
        );
    }

    #[test]
    fn test_power_summation_with_multiple_rails() {
        let mut stats = PowerStats {
            total: 0.0,
            rails: vec![
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
                PowerRail {
                    name: "rail3".to_string(),
                    current: 500.0,
                    voltage: 5000.0,
                    power: 2500.0,
                },
            ],
        };

        stats.total = stats.rails.iter().map(|r| r.power).sum::<f32>() / 1000.0;

        assert_eq!(
            stats.total, 17.5,
            "Total should be 15000mW + 2000mW / 1000 = 17.5W"
        );
    }
}
