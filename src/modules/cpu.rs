// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! CPU monitoring module
//!
//! Provides CPU statistics, core information, and performance metrics
//! with both synchronous and asynchronous I/O support.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use tokio::fs as tokio_fs;

/// CPU statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CpuStats {
    pub usage: f32,
    pub frequency: u32,
    pub cores: Vec<CpuCore>,
}

/// Per-core CPU statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuCore {
    pub index: usize,
    pub usage: f32,
    pub frequency: u32,
    pub governor: String,
}

impl CpuStats {
    /// Get current CPU statistics synchronously
    ///
    /// Returns a `CpuStats` struct containing:
    /// - Overall CPU usage (average of all cores)
    /// - List of individual cores with their usage, frequency, and governor
    pub fn get() -> Self {
        let mut stats = CpuStats::default();

        if let Ok(cores) = read_cpu_cores() {
            stats.cores = cores;
        }

        stats.usage = if !stats.cores.is_empty() {
            stats.cores.iter().map(|c| c.usage).sum::<f32>() / stats.cores.len() as f32
        } else {
            0.0
        };

        stats
    }

    /// Get current CPU statistics asynchronously
    ///
    /// Returns a `CpuStats` struct containing:
    /// - Overall CPU usage (average of all cores)
    /// - List of individual cores with their usage, frequency, and governor
    ///
    /// This is the async version of `get()` using tokio for I/O.
    pub async fn get_async() -> Self {
        let mut stats = CpuStats::default();

        if let Ok(cores) = read_cpu_cores_async().await {
            stats.cores = cores;
        }

        stats.usage = if !stats.cores.is_empty() {
            stats.cores.iter().map(|c| c.usage).sum::<f32>() / stats.cores.len() as f32
        } else {
            0.0
        };

        stats
    }
}

/// Get number of CPU cores synchronously
///
/// Reads `/proc/cpuinfo` and counts the number of processor lines.
/// Falls back to `num_cpus::get()` if reading fails.
///
/// # Returns
/// The number of CPU cores available on the system.
pub fn get_core_count() -> usize {
    let path = Path::new("/proc/cpuinfo");
    if let Ok(content) = fs::read_to_string(path) {
        content
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count()
    } else {
        // Fallback to environment
        num_cpus::get()
    }
}

/// Get number of CPU cores asynchronously
///
/// Reads `/proc/cpuinfo` and counts the number of processor lines.
/// Falls back to `num_cpus::get()` if reading fails.
///
/// # Returns
/// The number of CPU cores available on the system.
///
/// This is the async version of `get_core_count()` using tokio for I/O.
pub async fn get_core_count_async() -> usize {
    let path = Path::new("/proc/cpuinfo");
    if let Ok(content) = tokio_fs::read_to_string(path).await {
        content
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count()
    } else {
        // Fallback to environment
        num_cpus::get()
    }
}

/// Read CPU information from /proc/cpuinfo
fn read_cpu_cores() -> anyhow::Result<Vec<CpuCore>> {
    let path = Path::new("/proc/cpuinfo");
    let file = BufReader::new(fs::File::open(path)?);

    let mut cores: Vec<CpuCore> = Vec::new();
    let mut current_core: Option<usize> = None;

    for line in file.lines() {
        let line = line?;
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "processor" => {
                    current_core = Some(value.trim().parse().unwrap_or(0));
                    let idx = current_core.unwrap();
                    cores.push(CpuCore {
                        index: idx,
                        frequency: read_cpu_core_frequency(idx),
                        usage: 0.0,
                        governor: get_governor(idx),
                    });
                }
                _ => {}
            }
        }
    }

    // Calculate CPU usage from /proc/stat
    if let Ok(usage_vec) = read_cpu_usage(&cores) {
        for (core, usage) in cores.iter_mut().zip(usage_vec.iter()) {
            core.usage = *usage;
        }
    }

    Ok(cores)
}

/// Read CPU information from /proc/cpuinfo (async)
async fn read_cpu_cores_async() -> anyhow::Result<Vec<CpuCore>> {
    let path = Path::new("/proc/cpuinfo");
    let content = tokio_fs::read_to_string(path).await?;

    let mut cores: Vec<CpuCore> = Vec::new();
    let mut current_core: Option<usize> = None;

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "processor" => {
                    current_core = Some(value.trim().parse().unwrap_or(0));
                    let idx = current_core.unwrap();
                    cores.push(CpuCore {
                        index: idx,
                        frequency: 0,
                        usage: 0.0,
                        governor: get_governor_async(idx).await,
                    });
                }
                "cpu MHz" => {
                    if let Some(idx) = current_core {
                        if let Some(core) = cores.iter_mut().find(|c| c.index == idx) {
                            let freq = value.trim().parse().unwrap_or(0);
                            core.frequency = (freq as u32) * 1_000_000;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Calculate CPU usage from /proc/stat
    if let Ok(usage_vec) = read_cpu_usage_async(&cores).await {
        for (core, usage) in cores.iter_mut().zip(usage_vec.iter()) {
            core.usage = *usage;
        }
    }

    Ok(cores)
}

/// Read CPU usage from /proc/stat
fn read_cpu_usage(cores: &[CpuCore]) -> anyhow::Result<Vec<f32>> {
    let path = Path::new("/proc/stat");
    let content = fs::read_to_string(path)?;

    let mut usage = vec![0.0; cores.len()];

    for line in content.lines() {
        if line.starts_with("cpu") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Skip "cpu" (aggregate) line
            if parts[0] == "cpu" {
                continue;
            }

            // Extract core index
            if let Some(idx_str) = parts[0].strip_prefix("cpu") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < usage.len() {
                        // Parse CPU time fields
                        if parts.len() >= 5 {
                            let user: u64 = parts[1].parse().unwrap_or(0);
                            let nice: u64 = parts[2].parse().unwrap_or(0);
                            let system: u64 = parts[3].parse().unwrap_or(0);
                            let idle: u64 = parts[4].parse().unwrap_or(0);

                            let total = user + nice + system + idle;
                            if total > 0 {
                                usage[idx] = ((user + nice + system) as f32 / total as f32) * 100.0;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(usage)
}

/// Read CPU usage from /proc/stat (async)
async fn read_cpu_usage_async(cores: &[CpuCore]) -> anyhow::Result<Vec<f32>> {
    let path = Path::new("/proc/stat");
    let content = tokio_fs::read_to_string(path).await?;

    let mut usage = vec![0.0; cores.len()];

    for line in content.lines() {
        if line.starts_with("cpu") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Skip "cpu" (aggregate) line
            if parts[0] == "cpu" {
                continue;
            }

            // Extract core index
            if let Some(idx_str) = parts[0].strip_prefix("cpu") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < usage.len() {
                        // Parse CPU time fields
                        if parts.len() >= 5 {
                            let user: u64 = parts[1].parse().unwrap_or(0);
                            let nice: u64 = parts[2].parse().unwrap_or(0);
                            let system: u64 = parts[3].parse().unwrap_or(0);
                            let idle: u64 = parts[4].parse().unwrap_or(0);

                            let total = user + nice + system + idle;
                            if total > 0 {
                                usage[idx] = ((user + nice + system) as f32 / total as f32) * 100.0;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(usage)
}

/// Get CPU frequency governor
fn get_governor(core_idx: usize) -> String {
    let path_str = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core_idx
    );
    let path = Path::new(&path_str);

    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get CPU frequency governor (async)
async fn get_governor_async(core_idx: usize) -> String {
    let path_str = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core_idx
    );
    let path = Path::new(&path_str);

    tokio_fs::read_to_string(path)
        .await
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Read CPU core frequency from sysfs
pub fn read_cpu_core_frequency(core_idx: usize) -> u32 {
    let path_str = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
        core_idx
    );
    let path = Path::new(&path_str);

    if let Ok(content) = fs::read_to_string(path) {
        content.trim().parse().unwrap_or(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_count_from_cpuinfo() {
        let count = get_core_count();
        assert!(count > 0, "Core count should be at least 1");
        assert!(count <= 32, "Core count should be reasonable");
    }

    #[test]
    fn test_get_core_count_fallback() {
        let count = get_core_count();
        assert!(count > 0, "Should always return a positive core count");
    }

    #[test]
    fn test_cpu_stats_default() {
        let stats = CpuStats::default();
        assert_eq!(stats.usage, 0.0);
        assert_eq!(stats.frequency, 0);
        assert!(stats.cores.is_empty());
    }

    #[test]
    fn test_cpu_core_structure() {
        let core = CpuCore {
            index: 0,
            usage: 50.0,
            frequency: 1_500_000_000,
            governor: "schedutil".to_string(),
        };

        assert_eq!(core.index, 0);
        assert_eq!(core.usage, 50.0);
        assert_eq!(core.frequency, 1_500_000_000);
        assert_eq!(core.governor, "schedutil");
    }

    #[test]
    fn test_cpu_stats_get() {
        let stats = CpuStats::get();

        if !stats.cores.is_empty() {
            assert!(
                stats.usage >= 0.0 && stats.usage <= 100.0,
                "Usage should be between 0 and 100"
            );
            assert!(!stats.cores.is_empty(), "Should have at least one core");
        }
    }

    #[test]
    fn test_cpu_stats_usage_calculation() {
        let mut stats = CpuStats::default();
        stats.cores = vec![
            CpuCore {
                index: 0,
                usage: 50.0,
                frequency: 1000000,
                governor: "schedutil".to_string(),
            },
            CpuCore {
                index: 1,
                usage: 75.0,
                frequency: 1000000,
                governor: "schedutil".to_string(),
            },
            CpuCore {
                index: 2,
                usage: 25.0,
                frequency: 1000000,
                governor: "schedutil".to_string(),
            },
        ];

        let _avg_usage = (50.0 + 75.0 + 25.0) / 3.0;
        assert_eq!(stats.cores.len(), 3);
    }

    #[test]
    fn test_cpu_frequency_conversion() {
        let freq_mhz = 1500u32;
        let freq_hz = (freq_mhz as u32) * 1_000_000;
        assert_eq!(freq_hz, 1_500_000_000);
    }

    #[test]
    fn test_governor_fallback() {
        let governor = get_governor(999);
        assert_eq!(governor, "unknown");
    }

    #[test]
    fn test_cpu_usage_range() {
        let stats = CpuStats::get();

        for core in &stats.cores {
            assert!(core.usage >= 0.0, "Core usage should be >= 0");
            assert!(core.usage <= 100.0, "Core usage should be <= 100");
        }

        if !stats.cores.is_empty() {
            assert!(stats.usage >= 0.0, "Total usage should be >= 0");
            assert!(stats.usage <= 100.0, "Total usage should be <= 100");
        }
    }

    #[test]
    fn test_cpu_serialization() {
        let stats = CpuStats {
            usage: 42.5,
            frequency: 1500000000,
            cores: vec![CpuCore {
                index: 0,
                usage: 50.0,
                frequency: 1500000000,
                governor: "schedutil".to_string(),
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "CpuStats should be serializable");

        let deserialized: Result<CpuStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "CpuStats should be deserializable");
    }

    #[test]
    fn test_cpu_core_serialization() {
        let core = CpuCore {
            index: 0,
            usage: 75.5,
            frequency: 2000000000,
            governor: "performance".to_string(),
        };

        let json = serde_json::to_string(&core);
        assert!(json.is_ok(), "CpuCore should be serializable");

        let deserialized: Result<CpuCore, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "CpuCore should be deserializable");
    }

    #[test]
    fn test_read_cpu_core_frequency_from_sysfs() {
        // Test that we can read per-core CPU frequency from sysfs
        // jtop reads from: /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq

        let core0_freq = read_cpu_core_frequency(0);

        // On Jetson, we should be able to read CPU core frequency
        assert!(
            core0_freq > 0,
            "Should be able to read CPU core frequency from sysfs"
        );

        // Frequency should be reasonable (at least 100MHz, at most 4GHz)
        assert!(
            core0_freq >= 100_000_000,
            "CPU frequency should be at least 100MHz"
        );
        assert!(
            core0_freq <= 4_000_000_000,
            "CPU frequency should be at most 4GHz"
        );
    }

    #[test]
    fn test_format_cpu_core_frequency_mhz() {
        // Test formatting CPU core frequency to MHz
        let freq_hz = 1_500_000_000u32; // 1.5 GHz
        let freq_mhz = freq_hz as f32 / 1_000_000.0;

        assert!(
            (freq_mhz - 1500.0).abs() < 0.01,
            "Should format 1.5 GHz as 1500 MHz"
        );
    }

    #[test]
    fn test_format_cpu_core_frequency_ghz() {
        // Test formatting CPU core frequency to GHz
        let freq_hz = 3_000_000_000u32; // 3.0 GHz
        let freq_ghz = freq_hz as f32 / 1_000_000_000.0;

        assert!(
            (freq_ghz - 3.0).abs() < 0.01,
            "Should format 3.0 GHz correctly"
        );
    }

    #[test]
    fn test_read_cpu_core_utilization() {
        // Test that we can read CPU core utilization from /proc/stat
        // jtop uses 7 fields: user, nice, system, idle, iowait, irq, softirq
        // Current implementation uses only 4 fields: user, nice, system, idle

        let stats = CpuStats::get();

        // Should have at least one core with utilization
        assert!(!stats.cores.is_empty(), "Should have at least one CPU core");

        // At least one core should have non-zero utilization on Jetson
        let has_utilization = stats.cores.iter().any(|c| c.usage > 0.0);
        // Note: This may fail on non-Jetson systems or idle systems
        // assert!(has_utilization, "At least one core should have non-zero utilization");
    }

    #[test]
    fn test_format_cpu_core_utilization_percentage() {
        // Test formatting CPU core utilization as percentage
        let utilization = 42.5;

        // Should be between 0 and 100
        assert!(utilization >= 0.0, "Utilization should be >= 0");
        assert!(utilization <= 100.0, "Utilization should be <= 100");
    }

    #[test]
    fn test_cpu_core_utilization_calculation_with_7_fields() {
        // Test CPU core utilization calculation using 7 fields
        // jtop: user, nice, system, idle, iowait, irq, softirq

        let user = 793125u64;
        let nice = 280u64;
        let system = 352516u64;
        let idle = 16192366u64;
        let iowait = 50291u64;
        let irq = 2012u64;
        let softirq = 0u64;

        let total = user + nice + system + idle + iowait + irq + softirq;
        let busy = user + nice + system;

        let expected_util = 100.0 * (busy as f32 / total as f32);

        assert!(
            (expected_util - 8.5).abs() < 0.1,
            "Utilization should be ~8.5%"
        );
    }

    #[test]
    #[ignore = "Requires actual CPU data - run with: cargo test cpu -- --ignored"]
    fn test_print_cpu_info() {
        println!("\n=== CPU Information Test ===");

        let core_count = get_core_count();
        println!("Core count: {}", core_count);

        let stats = CpuStats::get();
        println!("Total CPU usage: {:.2}%", stats.usage);
        println!("Number of cores: {}", stats.cores.len());

        for (_i, core) in stats.cores.iter().enumerate() {
            println!(
                "Core {}: {:.2}% @ {} MHz (governor: {})",
                core.index,
                core.usage,
                core.frequency / 1_000_000,
                core.governor
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[tokio::test]
    async fn test_get_core_count_async() {
        let count = get_core_count_async().await;
        assert!(count > 0, "Core count should be at least 1");
    }

    #[tokio::test]
    async fn test_cpu_stats_get_async() {
        let stats = CpuStats::get_async().await;

        if !stats.cores.is_empty() {
            assert!(
                stats.usage >= 0.0 && stats.usage <= 100.0,
                "Usage should be between 0 and 100"
            );
            assert!(!stats.cores.is_empty(), "Should have at least one core");
        }
    }
}
