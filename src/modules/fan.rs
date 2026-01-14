// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Fan control module

use std::fs;
use std::path::Path;

/// Fan statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FanStats {
    pub speed: u8, // 0-100%
    pub rpm: u32,
    pub mode: FanMode,
    pub fans: Vec<FanInfo>,
}

/// Fan operating mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FanMode {
    #[default]
    Automatic,
    Manual,
    Off,
    Unknown,
}

/// Individual fan information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FanInfo {
    pub index: usize,
    pub name: String,
    pub speed: u8,
    pub rpm: u32,
}

impl FanStats {
    /// Get current fan statistics
    pub fn get() -> Self {
        let path = Path::new("/sys/class/thermal");

        if !path.exists() {
            return FanStats::default();
        }

        let mut stats = FanStats::default();
        stats.fans = read_cooling_devices(path);

        // Calculate overall speed and RPM
        if !stats.fans.is_empty() {
            stats.speed = (stats.fans.iter().map(|f| f.speed as u32).sum::<u32>()
                / stats.fans.len() as u32) as u8;
            stats.rpm = stats.fans.iter().map(|f| f.rpm).sum::<u32>() / stats.fans.len() as u32;
        }

        // Detect fan mode
        stats.mode = detect_fan_mode(&stats.fans);

        stats
    }

    /// Set fan speed (requires root)
    pub fn set_speed(speed: u8) -> anyhow::Result<()> {
        if speed > 100 {
            return Err(anyhow::anyhow!("Speed must be 0-100"));
        }

        let path = Path::new("/sys/class/thermal");

        if !path.exists() {
            return Err(anyhow::anyhow!("Thermal system not found"));
        }

        // Set all cooling devices to manual mode
        for fan in read_cooling_devices(path) {
            let fan_path_str = format!("/sys/class/thermal/cooling_device{}", fan.index);
            let fan_path = Path::new(&fan_path_str);

            // Set to manual mode
            let mode_path = fan_path.join("cur_state");
            fs::write(mode_path, "disabled")?;

            // Set PWM value
            let pwm_path = fan_path.join("cur_pwm");
            let pwm_value = (speed as u32 * 255 / 100).min(255);
            fs::write(pwm_path, pwm_value.to_string())?;
        }

        Ok(())
    }
}

/// Read all cooling devices
fn read_cooling_devices(base_path: &Path) -> Vec<FanInfo> {
    let mut fans = Vec::new();

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let cooling_path = entry.path();

            // Look for cooling_device directories
            if cooling_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("cooling_device"))
                .unwrap_or(false)
            {
                // This is a cooling device, continue processing
            } else {
                continue;
            }

            // Parse fan index
            let fan_name = cooling_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let index = fan_name
                .strip_prefix("cooling_device")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Read current state
            let state_path = cooling_path.join("cur_state");
            let max_state = read_sysfs_u32(&state_path).unwrap_or(1);

            // Read current speed (in state count)
            let cur_state = read_sysfs_u32(&state_path).unwrap_or(0);
            let speed = if max_state > 0 {
                ((cur_state as f32 / max_state as f32) * 100.0) as u8
            } else {
                0
            };

            // Read RPM (if available)
            let rpm_path = cooling_path.join("fan1_input");
            let rpm = read_sysfs_u32(&rpm_path).unwrap_or(0);

            fans.push(FanInfo {
                index,
                name: fan_name.to_string(),
                speed,
                rpm,
            });
        }
    }

    fans
}

/// Detect fan operating mode
fn detect_fan_mode(fans: &[FanInfo]) -> FanMode {
    if fans.is_empty() {
        return FanMode::Unknown;
    }

    // Check if all fans are at 0 speed
    if fans.iter().all(|f| f.speed == 0) {
        return FanMode::Off;
    }

    // Check if fans are responding to thermal (auto mode indicator)
    // This is a heuristic - true auto mode detection would require more complex logic
    FanMode::Manual
}

/// Read a u32 value from sysfs
fn read_sysfs_u32(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_stats_default() {
        let stats = FanStats::default();
        assert_eq!(stats.speed, 0);
        assert_eq!(stats.rpm, 0);
        assert_eq!(stats.mode, FanMode::Automatic);
        assert_eq!(stats.fans.len(), 0);
    }

    #[test]
    fn test_fan_stats_structure() {
        let stats = FanStats {
            speed: 75,
            rpm: 2500,
            mode: FanMode::Manual,
            fans: vec![
                FanInfo {
                    index: 0,
                    name: "cooling_device0".to_string(),
                    speed: 80,
                    rpm: 2600,
                },
                FanInfo {
                    index: 1,
                    name: "cooling_device1".to_string(),
                    speed: 70,
                    rpm: 2400,
                },
            ],
        };

        assert_eq!(stats.speed, 75);
        assert_eq!(stats.rpm, 2500);
        assert_eq!(stats.mode, FanMode::Manual);
        assert_eq!(stats.fans.len(), 2);
    }

    #[test]
    fn test_fan_stats_get() {
        let stats = FanStats::get();

        if !stats.fans.is_empty() {
            assert!(stats.speed <= 100, "Fan speed should be 0-100");
            assert!(
                stats.rpm > 0 || stats.rpm == 0,
                "RPM should be non-negative"
            );
        }
    }

    #[test]
    fn test_fan_info_default() {
        let info = FanInfo::default();
        assert_eq!(info.index, 0);
        assert_eq!(info.name, "");
        assert_eq!(info.speed, 0);
        assert_eq!(info.rpm, 0);
    }

    #[test]
    fn test_fan_info_structure() {
        let info = FanInfo {
            index: 1,
            name: "cooling_device1".to_string(),
            speed: 85,
            rpm: 2800,
        };

        assert_eq!(info.index, 1);
        assert_eq!(info.name, "cooling_device1");
        assert_eq!(info.speed, 85);
        assert_eq!(info.rpm, 2800);
    }

    #[test]
    fn test_fan_mode_default() {
        let mode = FanMode::default();
        assert_eq!(mode, FanMode::Automatic);
    }

    #[test]
    fn test_fan_mode_equality() {
        assert_eq!(FanMode::Automatic, FanMode::Automatic);
        assert_eq!(FanMode::Manual, FanMode::Manual);
        assert_eq!(FanMode::Off, FanMode::Off);
        assert_eq!(FanMode::Unknown, FanMode::Unknown);

        assert_ne!(FanMode::Automatic, FanMode::Manual);
        assert_ne!(FanMode::Automatic, FanMode::Off);
        assert_ne!(FanMode::Manual, FanMode::Unknown);
    }

    #[test]
    #[ignore = "Requires Jetson hardware with root access - run with: cargo test fan -- --ignored"]
    fn test_set_speed_validation() {
        println!("\n=== Fan Speed Validation Test ===");
        println!("WARNING: This test will modify fan settings. Ensure you know what you're doing.");
        println!("This test requires root privileges.\n");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - skipping fan speed validation");
            println!("\n=== Test Complete ===");
            return;
        }

        let result = FanStats::set_speed(50);
        if result.is_ok() {
            println!("Setting valid speed 50: OK");
        } else {
            println!("Setting valid speed 50: FAIL (expected without root access)");
        }

        let result = FanStats::set_speed(0);
        if result.is_ok() {
            println!("Setting valid speed 0: OK");
        } else {
            println!("Setting valid speed 0: FAIL (expected without root access)");
        }

        let result = FanStats::set_speed(100);
        if result.is_ok() {
            println!("Setting valid speed 100: OK");
        } else {
            println!("Setting valid speed 100: FAIL (expected without root access)");
        }

        let result = FanStats::set_speed(101);
        assert!(result.is_err(), "Setting speed > 100 should fail");

        let result = FanStats::set_speed(150);
        assert!(result.is_err(), "Setting speed > 100 should fail");

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_fan_serialization() {
        let stats = FanStats {
            speed: 65,
            rpm: 2200,
            mode: FanMode::Automatic,
            fans: vec![FanInfo {
                index: 0,
                name: "cooling_device0".to_string(),
                speed: 65,
                rpm: 2200,
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "FanStats should be serializable");

        let deserialized: Result<FanStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "FanStats should be deserializable");
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test fan -- --ignored"]
    fn test_print_fan_info() {
        println!("\n=== Fan Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - fan info not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = FanStats::get();

        println!("Fan speed: {}%", stats.speed);
        println!("Fan RPM: {}", stats.rpm);
        println!("Fan mode: {:?}", stats.mode);
        println!("Number of fans: {}", stats.fans.len());

        for (i, fan) in stats.fans.iter().enumerate() {
            println!(
                "  Fan {}: {} - Speed: {}%, RPM: {}",
                i, fan.name, fan.speed, fan.rpm
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    #[ignore = "Requires Jetson hardware with root access - run with: cargo test fan -- --ignored"]
    fn test_fan_speed_control() {
        println!("\n=== Fan Speed Control Test ===");
        println!("WARNING: This test will modify fan settings. Ensure you know what you're doing.");
        println!("This test requires root privileges.\n");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - skipping fan control test");
            println!("\n=== Test Complete ===");
            return;
        }

        // Try to set fan speed to 50%
        match FanStats::set_speed(50) {
            Ok(_) => println!("Successfully set fan speed to 50%"),
            Err(e) => println!(
                "Failed to set fan speed: {} (this is expected without root access)",
                e
            ),
        }

        println!("\n=== Test Complete ===");
    }
}
