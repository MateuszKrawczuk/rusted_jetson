// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Fan control module

use std::fs;
use std::path::Path;

/// Fan statistics
#[derive(Debug, Clone, Default)]
pub struct FanStats {
    pub speed: u8,      // 0-100%
    pub rpm: u32,
    pub mode: FanMode,
    pub fans: Vec<FanInfo>,
}

/// Fan operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    Automatic,
    Manual,
    Off,
    Unknown,
}

/// Individual fan information
#[derive(Debug, Clone, Default)]
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
            stats.speed = stats.fans.iter().map(|f| f.speed as u32).sum::<u32>() / stats.fans.len() as u32 as u8;
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
            let fan_path = Path::new(&format!(
                "/sys/class/thermal/cooling_device{}",
                fan.index
            ));

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
            if !cooling_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("cooling_device"))
                .unwrap_or(false)
            {
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
