// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Memory monitoring module

use std::fs;
use std::path::Path;

/// Memory statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
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
