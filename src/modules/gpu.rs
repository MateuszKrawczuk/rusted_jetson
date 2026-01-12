// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! GPU monitoring module
//!
//! Provides GPU statistics including usage, frequency, temperature, and governor information
//! using sysfs devfreq interface or NVML for NVIDIA Jetson devices.

use std::fs;
use std::path::Path;

#[cfg(feature = "nvml")]
use nvml_wrapper as nvml;

use crate::modules::hardware::detect_board;

/// GPU statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct GpuStats {
    pub usage: f32,
    pub frequency: u32,
    pub temperature: f32,
    pub governor: String,
}

impl GpuStats {
    /// Get current GPU statistics
    ///
    /// Returns a `GpuStats` struct containing:
    /// - GPU usage percentage
    /// - GPU frequency (in Hz)
    /// - GPU temperature (in Celsius)
    /// - GPU governor (performance, powersave, etc.)
    ///
    /// Reads from `/sys/class/devfreq` for frequency and governor,
    /// `/sys/class/thermal` for temperature.
    ///
    /// Supports NVIDIA Thor (tegra264) via `gpu-gpc-0` and `gpu-nvd-0` devfreq paths.
    ///
    /// For JetPack 7.0+ (Thor), uses NVML if available for more accurate statistics.
    pub fn get() -> Self {
        let mut stats = GpuStats::default();

        #[cfg(feature = "nvml")]
        {
            // Check if we should use NVML (JetPack 7.0+)
            if should_use_nvml() {
                if let Ok(nvml_stats) = get_nvml_stats() {
                    return nvml_stats;
                }
            }
        }

        // Try to read from devfreq
        if let Some(devfreq_path) = find_gpu_devfreq() {
            stats.frequency = read_gpu_freq(&devfreq_path);
            stats.governor = read_gpu_governor(&devfreq_path);
            stats.usage = read_gpu_usage(&devfreq_path);
        }

        // Try to read temperature
        stats.temperature = read_gpu_temp();

        stats
    }
}

/// Find GPU devfreq path
fn find_gpu_devfreq() -> Option<String> {
    let base_path = Path::new("/sys/class/devfreq");

    if !base_path.exists() {
        return None;
    }

    // Known GPU devfreq paths
    let candidates = [
        "gpu-gpc-0", // Thor GPC
        "gpu-nvd-0", // Thor NVD
        "gpu",       // Generic
    ];

    for candidate in &candidates {
        let path = base_path.join(candidate);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    None
}

/// Read GPU frequency (in Hz)
fn read_gpu_freq(devfreq_path: &str) -> u32 {
    let path = Path::new(devfreq_path).join("cur_freq");

    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Read GPU governor
fn read_gpu_governor(devfreq_path: &str) -> String {
    let path = Path::new(devfreq_path).join("governor");

    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Read GPU usage (estimated from devfreq load)
fn read_gpu_usage(devfreq_path: &str) -> f32 {
    let load_path = Path::new(devfreq_path).join("device/load");

    // Some devices expose GPU load
    if let Ok(load_str) = fs::read_to_string(load_path) {
        if let Ok(load) = load_str.trim().parse::<u64>() {
            // Load is typically in 0-255 range, convert to percentage
            return (load as f32 / 255.0 * 100.0).min(100.0);
        }
    }

    // Fallback: estimate from frequency
    let freq = read_gpu_freq(devfreq_path);
    if freq > 0 {
        // Rough estimate: higher freq = more usage
        // This is not accurate, but better than 0
        let max_freq = read_gpu_max_freq(devfreq_path);
        if max_freq > 0 {
            return (freq as f32 / max_freq as f32 * 100.0).min(100.0);
        }
    }

    0.0
}

/// Read GPU maximum frequency
///
/// Reads the maximum frequency from the specified devfreq path.
///
/// # Arguments
/// * `devfreq_path` - Path to the GPU devfreq directory (e.g., "/sys/class/devfreq/gpu")
///
/// # Returns
/// Maximum GPU frequency in Hz, or 0 if unavailable.
pub fn read_gpu_max_freq(devfreq_path: &str) -> u32 {
    let path = Path::new(devfreq_path).join("max_freq");

    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_stats_default() {
        let stats = GpuStats::default();
        assert_eq!(stats.usage, 0.0);
        assert_eq!(stats.frequency, 0);
        assert_eq!(stats.temperature, 0.0);
        assert_eq!(stats.governor, "");
    }

    #[test]
    fn test_gpu_stats_structure() {
        let stats = GpuStats {
            usage: 75.5,
            frequency: 1_500_000_000,
            temperature: 65.0,
            governor: "performance".to_string(),
        };

        assert_eq!(stats.usage, 75.5);
        assert_eq!(stats.frequency, 1_500_000_000);
        assert_eq!(stats.temperature, 65.0);
        assert_eq!(stats.governor, "performance");
    }

    #[test]
    fn test_gpu_stats_get() {
        let stats = GpuStats::get();

        assert!(
            stats.usage >= 0.0 && stats.usage <= 100.0,
            "GPU usage should be between 0 and 100"
        );
        assert!(
            stats.temperature >= 0.0 && stats.temperature < 120.0,
            "GPU temperature should be reasonable (0-120°C)"
        );
    }

    #[test]
    fn test_find_gpu_devfreq() {
        let devfreq_path = find_gpu_devfreq();

        if let Some(path_str) = devfreq_path {
            let path = Path::new(&path_str);
            assert!(path.exists(), "Devfreq path should exist");
        }
    }

    #[test]
    fn test_gpu_frequency_range() {
        let stats = GpuStats::get();

        if stats.frequency > 0 {
            assert!(
                stats.frequency >= 100_000_000,
                "GPU frequency should be at least 100MHz"
            );
            assert!(
                stats.frequency <= 3_000_000_000,
                "GPU frequency should be at most 3GHz"
            );
        }
    }

    #[test]
    fn test_gpu_usage_calculation() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let usage = read_gpu_usage(&devfreq_path.unwrap());
            assert!(
                usage >= 0.0 && usage <= 100.0,
                "GPU usage should be between 0 and 100"
            );
        }
    }

    #[test]
    fn test_gpu_temperature_range() {
        let temp = read_gpu_temp();

        if temp > 0.0 {
            assert!(temp >= 20.0, "GPU temperature should be at least 20°C");
            assert!(temp < 120.0, "GPU temperature should be less than 120°C");
        }
    }

    #[test]
    fn test_gpu_governor() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let governor = read_gpu_governor(&devfreq_path.unwrap());
            assert!(!governor.is_empty(), "Governor should not be empty");
        }
    }

    #[test]
    fn test_gpu_serialization() {
        let stats = GpuStats {
            usage: 85.5,
            frequency: 2_000_000_000,
            temperature: 70.0,
            governor: "performance".to_string(),
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "GpuStats should be serializable");

        let deserialized: Result<GpuStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "GpuStats should be deserializable");
    }

    #[test]
    fn test_read_gpu_max_freq() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let max_freq = read_gpu_max_freq(&devfreq_path.unwrap());

            if max_freq > 0 {
                assert!(
                    max_freq >= 100_000_000,
                    "Max freq should be at least 100MHz"
                );
            }
        }
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test gpu -- --ignored"]
    fn test_print_gpu_info() {
        println!("\n=== GPU Information Test ===");

        let stats = GpuStats::get();
        println!("GPU usage: {:.2}%", stats.usage);
        println!("GPU frequency: {} MHz", stats.frequency / 1_000_000);
        println!("GPU temperature: {:.1}°C", stats.temperature);
        println!("GPU governor: {}", stats.governor);

        if let Some(devfreq_path) = find_gpu_devfreq() {
            println!("\nDevfreq path: {}", devfreq_path);
            println!(
                "Max frequency: {} MHz",
                read_gpu_max_freq(&devfreq_path) / 1_000_000
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_nvidia_thor_support() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let path_str = devfreq_path.unwrap();
            let is_thor = path_str.contains("gpu-gpc-0") || path_str.contains("gpu-nvd-0");

            if is_thor {
                println!("NVIDIA Thor GPU detected via devfreq path: {}", path_str);
            }
        }
    }

    #[cfg(feature = "nvml")]
    #[test]
    fn test_nvml_support() {
        #[allow(unused_imports)]
        use crate::modules::hardware::detect_board;

        let board = detect_board();

        if board.l4t.starts_with("38.") || board.l4t.starts_with("39.") {
            println!(
                "L4T {} detected (JetPack 7.0+), NVML should be available",
                board.l4t
            );
        }
    }
}

/// Read GPU temperature
fn read_gpu_temp() -> f32 {
    // Try thermal zones
    let thermal_base = Path::new("/sys/class/thermal");

    if !thermal_base.exists() {
        return 0.0;
    }

    // Search for GPU thermal zone
    if let Ok(entries) = fs::read_dir(thermal_base) {
        for entry in entries.flatten() {
            let zone_path = entry.path();
            let type_path = zone_path.join("type");

            if let Ok(zone_type) = fs::read_to_string(type_path) {
                if zone_type.contains("GPU") {
                    // Found GPU thermal zone
                    let temp_path = zone_path.join("temp");
                    if let Ok(temp_str) = fs::read_to_string(temp_path) {
                        // Temperature is in millidegrees Celsius
                        if let Ok(temp_milli) = temp_str.trim().parse::<i32>() {
                            return temp_milli as f32 / 1000.0;
                        }
                    }

                    #[cfg(feature = "nvml")]
                    fn should_use_nvml() -> bool {
                        // Check if JetPack 7.0 or newer by reading L4T version
                        let board = detect_board();

                        // Parse L4T version to get major.minor
                        // L4T format: "36.4.0" or "38.2.0"
                        let parts: Vec<&str> = board.l4t.split('.').collect();
                        if parts.len() >= 2 {
                            if let Ok(major) = parts[0].parse::<u32>() {
                                if let Ok(minor) = parts[1].parse::<u32>() {
                                    // L4T 36.x corresponds to JetPack 6.x
                                    // L4T 38.x corresponds to JetPack 7.x
                                    // So L4T >= 38.0 means JetPack 7.0+
                                    return major > 38 || (major == 38 && minor >= 0);
                                }
                            }
                        }

                        false
                    }

                    #[cfg(feature = "nvml")]
                    fn get_nvml_stats() -> anyhow::Result<GpuStats> {
                        let mut stats = GpuStats::default();

                        // Initialize NVML
                        nvml::nvmlInit()?;

                        // Get device count
                        let device_count = nvml::nvmlDeviceGetCount()?;

                        if device_count == 0 {
                            nvml::nvmlShutdown()?;
                            anyhow::bail!("No NVML devices found");
                        }

                        // Get first device
                        let device = nvml::nvmlDeviceGetHandleByIndex(0)?;

                        // Get utilization
                        let utilization = nvml::nvmlDeviceGetUtilizationRates(device)?;
                        stats.usage = utilization.gpu as f32;

                        // Get temperature
                        let temp =
                            nvml::nvmlDeviceGetTemperature(device, nvml::NVML_TEMPERATURE_GPU)?;
                        stats.temperature = temp as f32;

                        // Get clock info (SM clock)
                        let clock_info = nvml::nvmlDeviceGetClockInfo(device, nvml::NVML_CLOCK_SM)?;
                        stats.frequency = clock_info.clock as u32;

                        // Governor is always "nvml" when using NVML
                        stats.governor = "nvml".to_string();

                        // Shutdown NVML
                        nvml::nvmlShutdown()?;

                        Ok(stats)
                    }
                }
            }
        }
    }

    0.0
}
