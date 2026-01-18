// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Engine monitoring module (APE, DLA, NVDEC, NVENC)

use std::fs;
use std::path::Path;

/// Engine statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EngineStats {
    pub ape: EngineStatus,
    pub dla0: EngineStatus,
    pub dla1: EngineStatus,
    pub nvdec: EngineStatus,
    pub nvenc: EngineStatus,
    pub nvjpg: EngineStatus,
}

/// Individual engine status
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EngineStatus {
    pub name: String,
    pub enabled: bool,
    pub usage: u8,
    pub clock: u32,
}

impl EngineStats {
    /// Get current engine statistics
    pub fn get() -> Self {
        let path = Path::new("/sys/class/devfreq");

        if !path.exists() {
            return EngineStats::default();
        }

        EngineStats {
            ape: read_engine_status(path, "ape"),
            dla0: read_engine_status(path, "dla0"),
            dla1: read_engine_status(path, "dla1"),
            nvdec: read_nvdec_status(path),
            nvenc: read_nvenc_status(path),
            nvjpg: read_nvjpg_status(path),
        }
    }
}

/// Read engine status from devfreq
fn read_engine_status(base_path: &Path, engine_name: &str) -> EngineStatus {
    let engine_path = base_path.join(engine_name);

    if !engine_path.exists() {
        return EngineStatus {
            name: engine_name.to_string(),
            ..Default::default()
        };
    }

    let enabled = engine_path.join("available_frequencies").exists();

    let clock = read_sysfs_u32(&engine_path, "cur_freq").unwrap_or(0);

    EngineStatus {
        name: engine_name.to_string(),
        enabled,
        usage: 0,
        clock,
    }
}

/// Read NVDEC engine status
fn read_nvdec_status(_base_path: &Path) -> EngineStatus {
    let engine_name = "nvdec";

    let usage_path = Path::new("/sys/kernel/nvdec_usage");
    let usage = if usage_path.exists() {
        read_sysfs_u32(usage_path, "usage").unwrap_or(0) as u8
    } else {
        0
    };

    EngineStatus {
        name: engine_name.to_string(),
        enabled: usage > 0,
        usage,
        clock: 0,
    }
}

/// Read NVENC engine status
fn read_nvenc_status(_base_path: &Path) -> EngineStatus {
    let engine_name = "nvenc";

    let usage_path = Path::new("/sys/kernel/nvenc_usage");
    let usage = if usage_path.exists() {
        read_sysfs_u32(usage_path, "usage").unwrap_or(0) as u8
    } else {
        0
    };

    EngineStatus {
        name: engine_name.to_string(),
        enabled: usage > 0,
        usage,
        clock: 0,
    }
}

/// Read NVJPG engine status
fn read_nvjpg_status(_base_path: &Path) -> EngineStatus {
    let engine_name = "nvjpg";

    let usage_path = Path::new("/sys/kernel/nvjpg_usage");
    let usage = if usage_path.exists() {
        read_sysfs_u32(usage_path, "usage").unwrap_or(0) as u8
    } else {
        0
    };

    EngineStatus {
        name: engine_name.to_string(),
        enabled: usage > 0,
        usage,
        clock: 0,
    }
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
    fn test_engine_stats_default() {
        let stats = EngineStats::default();
        assert!(!stats.ape.enabled);
        assert!(!stats.dla0.enabled);
        assert!(!stats.dla1.enabled);
        assert!(!stats.nvdec.enabled);
        assert!(!stats.nvenc.enabled);
        assert!(!stats.nvjpg.enabled);
    }

    #[test]
    fn test_engine_status_default() {
        let status = EngineStatus::default();
        assert_eq!(status.name, "");
        assert!(!status.enabled);
        assert_eq!(status.usage, 0);
        assert_eq!(status.clock, 0);
    }

    #[test]
    fn test_engine_status_structure() {
        let status = EngineStatus {
            name: "APE".to_string(),
            enabled: true,
            usage: 75,
            clock: 500000000,
        };

        assert_eq!(status.name, "APE");
        assert!(status.enabled);
        assert_eq!(status.usage, 75);
        assert_eq!(status.clock, 500000000);
    }

    #[test]
    fn test_engine_stats_structure() {
        let stats = EngineStats {
            ape: EngineStatus {
                name: "APE".to_string(),
                enabled: true,
                usage: 80,
                clock: 500000000,
            },
            dla0: EngineStatus {
                name: "DLA0".to_string(),
                enabled: true,
                usage: 60,
                clock: 300000000,
            },
            dla1: EngineStatus {
                name: "DLA1".to_string(),
                enabled: true,
                usage: 55,
                clock: 300000000,
            },
            nvdec: EngineStatus {
                name: "NVDEC".to_string(),
                enabled: true,
                usage: 70,
                clock: 0,
            },
            nvenc: EngineStatus {
                name: "NVENC".to_string(),
                enabled: true,
                usage: 65,
                clock: 0,
            },
            nvjpg: EngineStatus {
                name: "NVJPG".to_string(),
                enabled: true,
                usage: 50,
                clock: 0,
            },
        };

        assert!(stats.ape.enabled);
        assert!(stats.dla0.enabled);
        assert!(stats.nvdec.enabled);
    }

    #[test]
    fn test_ape_engine_status_reading() {
        let stats = EngineStats::get();
        assert_eq!(stats.ape.name, "ape");
    }

    #[test]
    fn test_dla_engine_status_reading() {
        let stats = EngineStats::get();
        assert_eq!(stats.dla0.name, "dla0");
        assert_eq!(stats.dla1.name, "dla1");
    }

    #[test]
    fn test_nvdec_engine_status_reading() {
        let stats = EngineStats::get();
        assert_eq!(stats.nvdec.name, "nvdec");
        assert!(stats.nvdec.usage <= 100);
    }

    #[test]
    fn test_nvenc_engine_status_reading() {
        let stats = EngineStats::get();
        assert_eq!(stats.nvenc.name, "nvenc");
        assert!(stats.nvenc.usage <= 100);
    }

    #[test]
    fn test_engine_clock_detection() {
        let stats = EngineStats::get();

        if stats.ape.enabled {
            assert!(stats.ape.clock > 0);
        }

        if stats.dla0.enabled {
            assert!(stats.dla0.clock > 0);
        }
    }

    #[test]
    fn test_engine_serialization() {
        let stats = EngineStats {
            ape: EngineStatus {
                name: "APE".to_string(),
                enabled: true,
                usage: 80,
                clock: 500000000,
            },
            dla0: EngineStatus {
                name: "DLA0".to_string(),
                enabled: true,
                usage: 60,
                clock: 300000000,
            },
            dla1: EngineStatus {
                name: "DLA1".to_string(),
                enabled: true,
                usage: 55,
                clock: 300000000,
            },
            nvdec: EngineStatus {
                name: "NVDEC".to_string(),
                enabled: true,
                usage: 70,
                clock: 0,
            },
            nvenc: EngineStatus {
                name: "NVENC".to_string(),
                enabled: true,
                usage: 65,
                clock: 0,
            },
            nvjpg: EngineStatus {
                name: "NVJPG".to_string(),
                enabled: true,
                usage: 50,
                clock: 0,
            },
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "EngineStats should be serializable");

        let deserialized: Result<EngineStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "EngineStats should be deserializable");
    }

    #[test]
    fn test_engine_status_serialization() {
        let status = EngineStatus {
            name: "APE".to_string(),
            enabled: true,
            usage: 80,
            clock: 500000000,
        };

        let json = serde_json::to_string(&status);
        assert!(json.is_ok(), "EngineStatus should be serializable");

        let deserialized: Result<EngineStatus, _> = serde_json::from_str(&json.unwrap());
        assert!(
            deserialized.is_ok(),
            "EngineStatus should be deserializable"
        );
    }

    #[test]
    fn test_engine_usage_range() {
        let stats = EngineStats::get();

        assert!(stats.ape.usage <= 100);
        assert!(stats.dla0.usage <= 100);
        assert!(stats.dla1.usage <= 100);
        assert!(stats.nvdec.usage <= 100);
        assert!(stats.nvenc.usage <= 100);
        assert!(stats.nvjpg.usage <= 100);
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test engine -- --ignored"]
    fn test_print_engine_info() {
        println!("\n=== Engine Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - engine info not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = EngineStats::get();

        println!(
            "APE: {} (enabled: {}, usage: {}%, clock: {} Hz)",
            stats.ape.name, stats.ape.enabled, stats.ape.usage, stats.ape.clock
        );
        println!(
            "DLA0: {} (enabled: {}, usage: {}%, clock: {} Hz)",
            stats.dla0.name, stats.dla0.enabled, stats.dla0.usage, stats.dla0.clock
        );
        println!(
            "DLA1: {} (enabled: {}, usage: {}%, clock: {} Hz)",
            stats.dla1.name, stats.dla1.enabled, stats.dla1.usage, stats.dla1.clock
        );
        println!(
            "NVDEC: {} (enabled: {}, usage: {}%)",
            stats.nvdec.name, stats.nvdec.enabled, stats.nvdec.usage
        );
        println!(
            "NVENC: {} (enabled: {}, usage: {}%)",
            stats.nvenc.name, stats.nvenc.enabled, stats.nvenc.usage
        );
        println!(
            "NVJPG: {} (enabled: {}, usage: {}%)",
            stats.nvjpg.name, stats.nvjpg.enabled, stats.nvjpg.usage
        );

        println!("\n=== Test Complete ===");
    }
}
