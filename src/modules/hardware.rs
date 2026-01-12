// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Hardware detection module

use std::fs;
use std::path::Path;

/// Jetson board information
#[derive(Debug, Clone, serde::Serialize)]
pub struct BoardInfo {
    pub model: String,
    pub jetpack: String,
    pub l4t: String,
    pub serial: String,
}

impl Default for BoardInfo {
    fn default() -> Self {
        Self {
            model: "Unknown Jetson Board".to_string(),
            jetpack: "Unknown".to_string(),
            l4t: "Unknown".to_string(),
            serial: "Unknown".to_string(),
        }
    }
}

/// Parse L4T version from nv_tegra_release content
pub fn parse_l4t_version(content: &str) -> String {
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "L4T_VERSION" {
                return value.trim().to_string();
            }
        }
    }
    String::new()
}

/// Parse Jetpack version from nv_tegra_release content
pub fn parse_jetpack_version(content: &str) -> String {
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "JETPACK_VERSION" {
                return value.trim().to_string();
            }
        }
    }
    String::new()
}

/// Detect board model from /sys/firmware/devicetree/base/model
pub fn detect_board_model() -> String {
    let model_path = Path::new("/sys/firmware/devicetree/base/model");

    if let Ok(model) = fs::read_to_string(model_path) {
        let model = model.trim_end_matches('\0').trim();
        if !model.is_empty() {
            return model.to_string();
        }
    }

    "Unknown Jetson Board".to_string()
}

/// Detect board model from compatible device tree strings
pub fn detect_model_from_compatible() -> String {
    let compatible_path = Path::new("/sys/firmware/devicetree/base/compatible");

    if let Ok(compatible) = fs::read_to_string(compatible_path) {
        for model_str in compatible.split('\0') {
            if model_str.is_empty() {
                continue;
            }

            if model_str.contains("nvidia,p3772") {
                return "Jetson Xavier NX".to_string();
            } else if model_str.contains("nvidia,p3668") {
                return "Jetson TX2 NX".to_string();
            } else if model_str.contains("nvidia,p3509") {
                return "Jetson Nano".to_string();
            } else if model_str.contains("nvidia,p3701") {
                return "Jetson AGX Orin".to_string();
            } else if model_str.contains("nvidia,p2888") {
                return "Jetson TX1".to_string();
            } else if model_str.contains("nvidia,p2972") {
                return "Jetson AGX Xavier".to_string();
            } else if model_str.contains("nvidia,tegra264") {
                return "Jetson Thor".to_string();
            }
        }
    }

    "Unknown Jetson Board".to_string()
}

/// Detect board serial number from device tree
pub fn detect_serial_number() -> String {
    let serial_path = Path::new("/sys/firmware/devicetree/base/serial-number");

    if let Ok(serial) = fs::read_to_string(serial_path) {
        let serial = serial.trim_end_matches('\0').trim();
        if !serial.is_empty() {
            return serial.to_string();
        }
    }

    "Unknown".to_string()
}

/// Detect SoC architecture/variant
pub fn detect_architecture() -> String {
    let machine_path = Path::new("/sys/firmware/devicetree/base/model");

    if let Ok(model) = fs::read_to_string(machine_path) {
        let model = model.to_lowercase();
        if model.contains("tegra264") {
            return "Thor (tegra264)".to_string();
        } else if model.contains("tegra234") {
            return "Orin (tegra234)".to_string();
        } else if model.contains("tegra194") {
            return "Xavier (tegra194)".to_string();
        } else if model.contains("tegra186") {
            return "TX2 (tegra186)".to_string();
        } else if model.contains("tegra210") {
            return "TX1 (tegra210)".to_string();
        }
    }

    "Unknown".to_string()
}

/// Detect board information from /etc/nv_tegra_release
pub fn detect_board() -> BoardInfo {
    let mut info = BoardInfo::default();

    let release_path = Path::new("/etc/nv_tegra_release");
    if let Ok(content) = fs::read_to_string(release_path) {
        info.l4t = parse_l4t_version(&content);
        info.jetpack = parse_jetpack_version(&content);

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "BOARD" => info.model = value.trim().to_string(),
                    "SERIAL_NUMBER" => info.serial = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    if info.model == "Unknown Jetson Board" || info.model.is_empty() {
        info.model = detect_board_model();
    }

    if info.model == "Unknown Jetson Board" || info.model.is_empty() {
        info.model = detect_model_from_compatible();
    }

    if info.serial == "Unknown" || info.serial.is_empty() {
        info.serial = detect_serial_number();
    }

    info
}

/// Check if running on a Jetson device
pub fn is_jetson() -> bool {
    Path::new("/etc/nv_tegra_release").exists() || Path::new("/sys/module/tegra_fuse").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l4t_version_parsing_r38() {
        let content = "L4T_VERSION=38.2.0\nJETPACK_VERSION=6.0.1\nBOARD=p3701";
        assert_eq!(parse_l4t_version(content), "38.2.0");
    }

    #[test]
    fn test_l4t_version_parsing_r32() {
        let content = "L4T_VERSION=32.7.2\nJETPACK_VERSION=4.6.4\nBOARD=t186ref";
        assert_eq!(parse_l4t_version(content), "32.7.2");
    }

    #[test]
    fn test_jetpack_version_parsing() {
        let content = "JETPACK_VERSION=6.0.1\nL4T_VERSION=36.3.0\nBOARD=p3701";
        assert_eq!(parse_jetpack_version(content), "6.0.1");
    }

    #[test]
    fn test_jetpack_version_missing() {
        let content = "L4T_VERSION=36.3.0\nBOARD=p3701";
        assert_eq!(parse_jetpack_version(content), "");
    }

    #[test]
    fn test_board_info_default() {
        let info = BoardInfo::default();
        assert_eq!(info.model, "Unknown Jetson Board");
        assert_eq!(info.jetpack, "Unknown");
        assert_eq!(info.l4t, "Unknown");
        assert_eq!(info.serial, "Unknown");
    }

    #[test]
    fn test_is_jetson_with_tegra_fuse() {
        let has_fuse = Path::new("/sys/module/tegra_fuse").exists();
        assert_eq!(
            is_jetson(),
            has_fuse || Path::new("/etc/nv_tegra_release").exists()
        );
    }
}
