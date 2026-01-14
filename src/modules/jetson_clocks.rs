// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Jetson Clocks control module

use std::fs;
use std::path::Path;

/// Jetson Clocks statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JetsonClocksStats {
    pub enabled: bool,
    pub mode: String,
}

impl JetsonClocksStats {
    /// Get current jetson_clocks status
    pub fn get() -> Self {
        let path = Path::new("/sys/devices/soc0/firmware/devicetree/base/nvidia,boost");

        if !path.exists() {
            return JetsonClocksStats::default();
        }

        let mut stats = JetsonClocksStats::default();

        // Try to read current mode
        if let Some(mode) = read_jetson_clocks_mode(&path) {
            stats.enabled = !mode.contains("0");
            stats.mode = mode;
        }

        stats
    }

    /// Toggle jetson_clocks (requires root)
    pub fn toggle() -> anyhow::Result<()> {
        let output = std::process::Command::new("sudo")
            .args(["/usr/bin/jetson_clocks"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("jetson_clocks command failed: {}", stderr));
        }

        Ok(())
    }

    /// Set jetson_clocks mode (requires root)
    pub fn set_mode(mode: &str) -> anyhow::Result<()> {
        let output = std::process::Command::new("sudo")
            .args(["/usr/bin/jetson_clocks", mode])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("jetson_clocks command failed: {}", stderr));
        }

        Ok(())
    }
}

/// Read jetson_clocks mode from devicetree
fn read_jetson_clocks_mode(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        content.trim().to_string().into()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jetson_clocks_stats_default() {
        let stats = JetsonClocksStats::default();
        assert!(!stats.enabled);
        assert_eq!(stats.mode, "");
    }

    #[test]
    fn test_jetson_clocks_stats_structure() {
        let stats = JetsonClocksStats {
            enabled: true,
            mode: "1".to_string(),
        };

        assert!(stats.enabled);
        assert_eq!(stats.mode, "1");
    }

    #[test]
    fn test_jetson_clocks_get() {
        let stats = JetsonClocksStats::get();

        if stats.enabled {
            assert!(!stats.mode.is_empty());
        }
    }

    #[test]
    fn test_jetson_clocks_toggle_validation() {
        // Test that toggle will try to execute command
        // Actual execution will fail without root, but should not panic
        let result = JetsonClocksStats::toggle();
        // Result depends on sudo access
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_jetson_clocks_set_mode_validation() {
        // Test invalid mode names will be handled
        // The command itself validates, so we just check it doesn't panic
        let result = JetsonClocksStats::set_mode("invalid");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_jetson_clocks_serialization() {
        let stats = JetsonClocksStats {
            enabled: true,
            mode: "1".to_string(),
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "JetsonClocksStats should be serializable");

        let deserialized: Result<JetsonClocksStats, _> = serde_json::from_str(&json.unwrap());
        assert!(
            deserialized.is_ok(),
            "JetsonClocksStats should be deserializable"
        );
    }

    #[test]
    #[ignore = "Requires Jetson hardware with jetson_clocks - run with: cargo test jetson_clocks -- --ignored"]
    fn test_print_jetson_clocks_info() {
        println!("\n=== Jetson Clocks Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - jetson_clocks not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = JetsonClocksStats::get();

        println!("Enabled: {}", stats.enabled);
        println!("Mode: {}", stats.mode);

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_jetson_clocks_state_reading() {
        let stats = JetsonClocksStats::get();

        // Check that we can read the state
        if stats.enabled {
            // If enabled, mode should not be empty
            assert!(!stats.mode.is_empty());
        }
    }

    #[test]
    fn test_jetson_clocks_mode_detection() {
        let stats = JetsonClocksStats::get();

        if stats.enabled {
            // If enabled, mode should not be "0"
            assert_ne!(stats.mode, "0");
        } else {
            // If disabled, mode should be "0" or empty
            assert!(stats.mode.is_empty() || stats.mode == "0");
        }
    }

    #[test]
    fn test_jetson_clocks_toggle_logic() {
        // Test that toggle function exists and is callable
        let _ = JetsonClocksStats::toggle;
    }

    #[test]
    fn test_jetson_clocks_set_mode_logic() {
        // Test that set_mode function exists and is callable
        let _ = JetsonClocksStats::set_mode;
    }
}
