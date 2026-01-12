// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! CPU monitoring module

use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// CPU statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuStats {
    pub usage: f32,
    pub frequency: u32,
    pub cores: Vec<CpuCore>,
}

/// Per-core CPU statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuCore {
    pub index: usize,
    pub usage: f32,
    pub frequency: u32,
    pub governor: String,
}

impl CpuStats {
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
}

/// Get number of CPU cores
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

/// Read CPU information from /proc/cpuinfo
fn read_cpu_cores() -> anyhow::Result<Vec<CpuCore>> {
    let path = Path::new("/proc/cpuinfo");
    let file = BufReader::new(fs::File::open(path)?);

    let mut cores = Vec::new();
    let mut current_core: Option<usize> = None;

    for line in file.lines() {
        let line = line?;
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "processor" => {
                    current_core = Some(value.trim().parse().unwrap_or(0));
                }
                "cpu MHz" => {
                    if let Some(idx) = current_core {
                        let freq = value.trim().parse().unwrap_or(0);
                        cores.push(CpuCore {
                            index: idx,
                            frequency: (freq as u32) * 1_000_000,
                            usage: 0.0,
                            governor: get_governor(idx),
                        });
                    }
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

/// Get CPU frequency governor
fn get_governor(core_idx: usize) -> String {
    let path = Path::new(format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        core_idx
    ));

    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
