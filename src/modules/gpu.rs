// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! GPU monitoring module

use std::fs;
use std::path::Path;

/// GPU statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuStats {
    pub usage: f32,
    pub frequency: u32,
    pub temperature: f32,
    pub governor: String,
}

impl GpuStats {
    /// Get current GPU statistics
    pub fn get() -> Self {
        let mut stats = GpuStats::default();

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
fn read_gpu_max_freq(devfreq_path: &str) -> u32 {
    let path = Path::new(devfreq_path).join("max_freq");

    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
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
                }
            }
        }
    }

    0.0
}
