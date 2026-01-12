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

/// Detect board information from /etc/nv_tegra_release
pub fn detect_board() -> BoardInfo {
    let mut info = BoardInfo::default();

    // Read /etc/nv_tegra_release
    if let Ok(content) = fs::read_to_string("/etc/nv_tegra_release") {
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "BOARD" => info.model = value.trim().to_string(),
                    "JETPACK_VERSION" => info.jetpack = value.trim().to_string(),
                    "L4T_VERSION" => info.l4t = value.trim().to_string(),
                    "SERIAL_NUMBER" => info.serial = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    // Fallback to device tree
    if info.model == "Unknown Jetson Board" {
        info.model = detect_model_from_devicetree();
    }

    info
}

/// Detect model from device tree
fn detect_model_from_devicetree() -> String {
    let compatible_path = Path::new("/sys/firmware/devicetree/base/compatible");

    if let Ok(compatible) = fs::read_to_string(compatible_path) {
        // Device tree compatible strings are null-separated
        for model_str in compatible.split('\0') {
            if model_str.is_empty() {
                continue;
            }

            // Map known model strings
            if model_str.contains("p3772") {
                return "Jetson Xavier NX".to_string();
            } else if model_str.contains("p3668") {
                return "Jetson TX2".to_string();
            } else if model_str.contains("p3509") {
                return "Jetson Nano".to_string();
            } else if model_str.contains("p3701") {
                return "Jetson AGX Xavier".to_string();
            } else if model_str.contains("p2888") {
                return "Jetson TX1".to_string();
            }
        }
    }

    "Unknown Jetson Board".to_string()
}

/// Check if running on a Jetson device
pub fn is_jetson() -> bool {
    Path::new("/etc/nv_tegra_release").exists() || Path::new("/sys/module/tegra_fuse").exists()
}
