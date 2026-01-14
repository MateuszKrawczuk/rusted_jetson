// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Memory monitoring module

use std::fs;
use std::path::Path;

/// Memory statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    pub ram_used: u64,
    pub ram_total: u64,
    pub ram_cached: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub swap_cached: u64,
    pub iram_used: u64,
    pub iram_total: u64,
    pub iram_lfb: u64,
}

impl MemoryStats {
    /// Get current memory statistics
    pub fn get() -> Self {
        let path = Path::new("/proc/meminfo");

        if let Ok(content) = fs::read_to_string(path) {
            parse_meminfo(&content)
        } else {
            MemoryStats::default()
        }
    }
}

/// Parse /proc/meminfo
fn parse_meminfo(content: &str) -> MemoryStats {
    let mut stats = MemoryStats::default();
    let mut meminfo = std::collections::HashMap::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            // Remove kB suffix and trim
            let value_str = value.trim().trim_end_matches(" kB").trim().to_string();

            if let Ok(value) = value_str.parse::<u64>() {
                meminfo.insert(key.trim(), value * 1024); // Convert to bytes
            }
        }
    }

    // Parse RAM
    stats.ram_total = *meminfo.get("MemTotal").unwrap_or(&0);
    let mem_free = *meminfo.get("MemFree").unwrap_or(&0);
    let mem_buffers = *meminfo.get("Buffers").unwrap_or(&0);
    let mem_cached = *meminfo.get("Cached").unwrap_or(&0);
    stats.ram_cached = mem_cached;

    // Calculate used RAM
    stats.ram_used = stats
        .ram_total
        .saturating_sub(mem_free + mem_buffers + mem_cached);

    // Parse SWAP
    stats.swap_total = *meminfo.get("SwapTotal").unwrap_or(&0);
    let swap_free = *meminfo.get("SwapFree").unwrap_or(&0);
    stats.swap_used = stats.swap_total.saturating_sub(swap_free);

    // Parse SwapCached
    stats.swap_cached = *meminfo.get("SwapCached").unwrap_or(&0);

    // Parse IRAM (if available)
    stats.iram_total = *meminfo.get("IramTotal").unwrap_or(&0);
    stats.iram_lfb = *meminfo.get("IramLfb").unwrap_or(&0);
    let iram_free = *meminfo.get("IramFree").unwrap_or(&0);
    stats.iram_used = stats.iram_total.saturating_sub(iram_free + stats.iram_lfb);

    stats
}

/// Read EMC (External Memory Controller) frequency
///
/// Reads EMC frequency from /sys/kernel/debug/clk/emc/clk_rate or similar paths.
///
/// # Returns
/// EMC frequency in Hz, or 0 if unavailable.
pub fn read_emc_frequency() -> u64 {
    let paths = [
        "/sys/kernel/debug/clk/emc/clk_rate",
        "/sys/kernel/debug/clk/parent_emc/clk_rate",
        "/sys/kernel/debug/clk/emc_clk_source/clk_rate",
        "/sys/devices/platform/host1x/15000000.tsec/15000000.tsec/emc_rate",
    ];

    for path in &paths {
        let path = Path::new(path);
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(freq) = content.trim().parse::<u64>() {
                return freq;
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.ram_used, 0);
        assert_eq!(stats.ram_total, 0);
        assert_eq!(stats.ram_cached, 0);
        assert_eq!(stats.swap_used, 0);
        assert_eq!(stats.swap_total, 0);
        assert_eq!(stats.swap_cached, 0);
        assert_eq!(stats.iram_used, 0);
        assert_eq!(stats.iram_total, 0);
        assert_eq!(stats.iram_lfb, 0);
    }

    #[test]
    fn test_memory_stats_structure() {
        let stats = MemoryStats {
            ram_used: 4_000_000_000,
            ram_total: 8_000_000_000,
            ram_cached: 2_000_000_000,
            swap_used: 1_000_000_000,
            swap_total: 4_000_000_000,
            swap_cached: 500_000_000,
            iram_used: 1_000_000,
            iram_total: 2_000_000,
            iram_lfb: 100_000,
        };

        assert_eq!(stats.ram_used, 4_000_000_000);
        assert_eq!(stats.ram_total, 8_000_000_000);
        assert_eq!(stats.ram_cached, 2_000_000_000);
        assert_eq!(stats.swap_used, 1_000_000_000);
        assert_eq!(stats.swap_total, 4_000_000_000);
        assert_eq!(stats.swap_cached, 500_000_000);
        assert_eq!(stats.iram_used, 1_000_000);
        assert_eq!(stats.iram_total, 2_000_000);
        assert_eq!(stats.iram_lfb, 100_000);
    }

    #[test]
    fn test_memory_stats_get() {
        let stats = MemoryStats::get();

        if stats.ram_total > 0 {
            assert!(
                stats.ram_used <= stats.ram_total,
                "RAM used should be less than or equal to total"
            );
            assert!(
                stats.ram_cached <= stats.ram_total,
                "RAM cached should be less than or equal to total"
            );
        }

        if stats.swap_total > 0 {
            assert!(
                stats.swap_used <= stats.swap_total,
                "Swap used should be less than or equal to total"
            );
        }

        if stats.iram_total > 0 {
            assert!(
                stats.iram_used <= stats.iram_total,
                "IRAM used should be less than or equal to total"
            );
        }
    }

    #[test]
    fn test_parse_meminfo() {
        let sample_meminfo = r#"MemTotal:        8192000 kB
MemFree:         4096000 kB
Buffers:          512000 kB
Cached:          2048000 kB
SwapTotal:       4096000 kB
SwapFree:        3072000 kB
SwapCached:       256000 kB
IramTotal:         20480 kB
IramFree:           5120 kB
IramLfb:           2560 kB"#;

        let stats = parse_meminfo(sample_meminfo);

        assert_eq!(stats.ram_total, 8192000 * 1024);
        assert_eq!(stats.ram_cached, 2048000 * 1024);

        let expected_used = 8192000 * 1024 - 4096000 * 1024 - 512000 * 1024 - 2048000 * 1024;
        assert_eq!(stats.ram_used, expected_used);

        assert_eq!(stats.swap_total, 4096000 * 1024);
        assert_eq!(stats.swap_used, 4096000 * 1024 - 3072000 * 1024);
        assert_eq!(stats.swap_cached, 256000 * 1024);

        assert_eq!(stats.iram_total, 20480 * 1024);
        assert_eq!(stats.iram_lfb, 2560 * 1024);

        let expected_iram_used = 20480 * 1024 - 5120 * 1024 - 2560 * 1024;
        assert_eq!(stats.iram_used, expected_iram_used);
    }

    #[test]
    fn test_parse_meminfo_no_iram() {
        let sample_meminfo = r#"MemTotal:        8192000 kB
MemFree:         4096000 kB
Buffers:          512000 kB
Cached:          2048000 kB
SwapTotal:       4096000 kB
SwapFree:        3072000 kB
SwapCached:       256000 kB"#;

        let stats = parse_meminfo(sample_meminfo);

        assert_eq!(stats.ram_total, 8192000 * 1024);
        assert_eq!(stats.swap_total, 4096000 * 1024);
        assert_eq!(stats.iram_total, 0);
        assert_eq!(stats.iram_used, 0);
        assert_eq!(stats.iram_lfb, 0);
    }

    #[test]
    fn test_parse_meminfo_invalid_format() {
        let invalid_meminfo = r#"MemTotal: invalid kB
MemFree:         4096000 kB
Cached:          not a number"#;

        let stats = parse_meminfo(invalid_meminfo);

        assert_eq!(stats.ram_total, 0);
        assert_eq!(stats.ram_cached, 0);
    }

    #[test]
    fn test_memory_serialization() {
        let stats = MemoryStats {
            ram_used: 4_000_000_000,
            ram_total: 8_000_000_000,
            ram_cached: 2_000_000_000,
            swap_used: 1_000_000_000,
            swap_total: 4_000_000_000,
            swap_cached: 500_000_000,
            iram_used: 1_000_000,
            iram_total: 2_000_000,
            iram_lfb: 100_000,
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "MemoryStats should be serializable");

        let deserialized: Result<MemoryStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "MemoryStats should be deserializable");
    }

    #[test]
    fn test_read_emc_frequency() {
        let freq = read_emc_frequency();

        if freq > 0 {
            assert!(
                freq >= 100_000_000,
                "EMC frequency should be at least 100MHz"
            );
            assert!(
                freq <= 4_000_000_000,
                "EMC frequency should be at most 4GHz"
            );
        }
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test memory -- --ignored"]
    fn test_print_memory_info() {
        println!("\n=== Memory Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        let stats = MemoryStats::get();

        println!("RAM Used: {} MB", stats.ram_used / 1_048_576);
        println!("RAM Total: {} MB", stats.ram_total / 1_048_576);
        println!("RAM Cached: {} MB", stats.ram_cached / 1_048_576);
        println!("Swap Used: {} MB", stats.swap_used / 1_048_576);
        println!("Swap Total: {} MB", stats.swap_total / 1_048_576);
        println!("Swap Cached: {} MB", stats.swap_cached / 1_048_576);

        if stats.iram_total > 0 {
            println!("IRAM Used: {} KB", stats.iram_used / 1024);
            println!("IRAM Total: {} KB", stats.iram_total / 1024);
            println!("IRAM LFB: {} KB", stats.iram_lfb / 1024);
        }

        let emc_freq = read_emc_frequency();
        if emc_freq > 0 {
            println!("EMC Frequency: {} MHz", emc_freq / 1_000_000);
        } else {
            println!("EMC Frequency: Not available");
        }

        println!("\n=== Test Complete ===");
    }
}
