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
        } else if line.starts_with('#') {
            let l4t = parse_l4t_from_comment(line, "");
            if !l4t.is_empty() {
                return l4t;
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
        } else if line.starts_with('#') {
            let jetpack = parse_jetpack_from_comment(line, "");
            if !jetpack.is_empty() {
                return jetpack;
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

    let compatible_path = Path::new("/sys/firmware/devicetree/base/compatible");
    if let Ok(compatible) = fs::read_to_string(compatible_path) {
        let compatible = compatible.to_lowercase();
        if compatible.contains("tegra264") {
            return "Thor (tegra264)".to_string();
        } else if compatible.contains("tegra234") {
            return "Orin (tegra234)".to_string();
        } else if compatible.contains("tegra194") {
            return "Xavier (tegra194)".to_string();
        } else if compatible.contains("tegra186") {
            return "TX2 (tegra186)".to_string();
        } else if compatible.contains("tegra210") {
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

        let mut found_board = false;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "BOARD" => {
                        info.model = value.trim().to_string();
                        found_board = true;
                    }
                    "SERIAL_NUMBER" => info.serial = value.trim().to_string(),
                    _ => {}
                }
            } else if line.starts_with('#') {
                info.l4t = parse_l4t_from_comment(line, &info.l4t);
                info.jetpack = parse_jetpack_from_comment(line, &info.jetpack);
                if !found_board {
                    if let Some(board) = parse_board_from_comment(line) {
                        info.model = board;
                        found_board = true;
                    }
                }
            }
        }
    }

    if (info.jetpack.is_empty() || info.jetpack == "Unknown") && !info.l4t.is_empty() {
        info.jetpack = derive_jetpack_from_l4t(&info.l4t);
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

/// Derive Jetpack version from L4T version
fn derive_jetpack_from_l4t(l4t: &str) -> String {
    use std::collections::HashMap;

    let parts: Vec<&str> = l4t.split('.').collect();
    if parts.len() < 2 {
        return "Unknown".to_string();
    }

    let key = if parts.len() >= 3 {
        format!("{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        format!("{}.{}", parts[0], parts[1])
    };

    let l4t_to_jetpack: HashMap<&str, &str> = [
        ("38.4.4", "7.1"),
        ("38.4.3", "7.1"),
        ("38.4.2", "7.1"),
        ("38.4.1", "7.1"),
        ("38.4", "7.1"),
        ("38.2.1", "7.0"),
        ("38.2", "7.0"),
        ("38.1", "7.0"),
        ("38.0", "7.0"),
        ("36.4.7", "6.2.1"),
        ("36.4.6", "6.2.1"),
        ("36.4.5", "6.2.1"),
        ("36.4.4", "6.2.1"),
        ("36.4.3", "6.2"),
        ("36.4.2", "6.2"),
        ("36.4.1", "6.2"),
        ("36.4", "6.1"),
        ("36.3", "6.0"),
        ("36.2", "6.0 DP"),
        ("36.1", "6.0"),
        ("36.0", "6.0"),
        ("35.6.2", "5.1.5"),
        ("35.6.1", "5.1.5"),
        ("35.6", "5.1.4"),
        ("35.5", "5.1.3"),
        ("35.4.1", "5.1.2"),
        ("35.4", "5.1.2"),
        ("35.3.1", "5.1.1"),
        ("35.3", "5.1.1"),
        ("35.2.1", "5.1"),
        ("35.2", "5.1"),
        ("35.1", "5.1"),
        ("35.0", "5.1"),
        ("34.1.1", "5.0.1 DP"),
        ("34.1", "5.0 DP"),
        ("34.0", "5.0 DP"),
        ("32.7.6", "4.6.6"),
        ("32.7.5", "4.6.5"),
        ("32.7.4", "4.6.4"),
        ("32.7.3", "4.6.3"),
        ("32.7.2", "4.6.2"),
        ("32.7.1", "4.6.1"),
        ("32.7", "4.6.x"),
        ("32.6.1", "4.6"),
        ("32.6", "4.6"),
        ("32.5.1", "4.5.1"),
        ("32.5", "4.5"),
        ("32.4.4", "4.4.1"),
        ("32.4.3", "4.4"),
        ("32.4.2", "4.4 DP"),
        ("32.4", "4.4"),
        ("32.3.1", "4.3"),
        ("32.3", "4.3"),
        ("32.2.1", "4.2.3"),
        ("32.2", "4.2.2"),
        ("32.1.1", "4.1.1 DP"),
        ("32.1", "4.1"),
        ("31.1", "4.1 DP"),
        ("31.0", "4.1 DP"),
        ("28.5", "3.3.4"),
        ("28.4", "3.3.3"),
        ("28.3.2", "3.3.2"),
        ("28.3.1", "3.3.1"),
        ("28.3", "3.3.2"),
        ("28.2.1", "3.3"),
        ("28.2", "3.2.1"),
        ("27.1", "3.0"),
        ("26.0", "3.0"),
        ("25.0", "3.0"),
        ("24.2.1", "2.3.1"),
        ("24.1", "2.3"),
        ("23.0", "2.3"),
        ("22.0", "2.3"),
        ("21.5", "2.3.1"),
        ("21.0", "2.3"),
    ]
    .into_iter()
    .collect();

    l4t_to_jetpack
        .get(key.as_str())
        .copied()
        .unwrap_or("Unknown")
        .to_string()
}

/// Parse L4T version from comment format like "# R36 (release), REVISION: 4.3"
fn parse_l4t_from_comment(line: &str, current_l4t: &str) -> String {
    if current_l4t.is_empty() && line.contains("R") {
        if let Some(start) = line.find('R') {
            let rest = &line[start + 1..];
            if let Some(end) = rest.find(' ') {
                let release_num = &rest[..end];
                let release_num: u32 = release_num.parse().unwrap_or(0);
                if release_num >= 20 {
                    if line.contains("REVISION:") {
                        if let Some(rev_start) = line.find("REVISION:") {
                            let rest = &line[rev_start + "REVISION:".len()..];
                            if let Some(rev_end) = rest.find(',') {
                                return format!("{}.{}", release_num, rest[..rev_end].trim());
                            }
                        }
                    }
                    return format!("{}.0", release_num);
                }
            }
        }
    }
    current_l4t.to_string()
}

/// Parse Jetpack version from comment line
fn parse_jetpack_from_comment(line: &str, current_jetpack: &str) -> String {
    if !current_jetpack.is_empty() {
        return current_jetpack.to_string();
    }
    if line.contains("JETPACK_VERSION=") {
        if let Some(start) = line.find("JETPACK_VERSION=") {
            let rest = &line[start + "JETPACK_VERSION=".len()..];
            let end = rest.find([',', ' ', '\n']).unwrap_or(rest.len());
            return rest[..end].trim().to_string();
        }
    }
    current_jetpack.to_string()
}

/// Parse board model from comment line like "BOARD: generic"
fn parse_board_from_comment(line: &str) -> Option<String> {
    if line.contains("BOARD:") {
        if let Some(start) = line.find("BOARD:") {
            let rest = &line[start + "BOARD:".len()..];
            let end = rest
                .find(',')
                .or_else(|| rest.find('\n'))
                .unwrap_or(rest.len());
            let board = rest[..end].trim();
            if board != "generic" && !board.is_empty() {
                return Some(board.to_string());
            }
        }
    }
    None
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

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test hardware -- --ignored"]
    fn test_print_hardware_info() {
        println!("\n=== Hardware Detection Test ===");

        let is_jetson_device = is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - limited info available");
        }

        let model = detect_board_model();
        println!("Board Model (from sysfs/model): {}", model);

        let model_compat = detect_model_from_compatible();
        println!("Board Model (from compatible): {}", model_compat);

        let serial = detect_serial_number();
        println!("Serial Number: {}", serial);

        let arch = detect_architecture();
        println!("Architecture: {}", arch);

        let board = detect_board();
        println!("\n=== Full Board Info ===");
        println!("Model: {}", board.model);
        println!("Jetpack: {}", board.jetpack);
        println!("L4T: {}", board.l4t);
        println!("Serial: {}", board.serial);

        let release_path = Path::new("/etc/nv_tegra_release");
        if let Ok(content) = fs::read_to_string(release_path) {
            println!("\n=== Debug: /etc/nv_tegra_release ===");
            println!("{}", content);
        } else {
            println!("\n=== Debug: /etc/nv_tegra_release not found ===");
        }

        println!("\n=== Test Complete ===");
    }
}
