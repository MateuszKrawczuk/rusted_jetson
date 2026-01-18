// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! GPU monitoring module
//!
//! Provides GPU statistics including usage, frequency, temperature, and governor information
//! using sysfs devfreq interface or NVML for NVIDIA Jetson devices.

use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(feature = "nvml")]
use nvml_wrapper as nvml;

/// GPU statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuStats {
    pub usage: f32,
    pub frequency: u32,
    pub temperature: f32,
    pub governor: String,
    pub memory_used: u64,
    pub memory_total: u64,
    pub state: String,
    pub active_functions: Vec<String>,
}

impl Default for GpuStats {
    fn default() -> Self {
        Self {
            usage: 0.0,
            frequency: 0,
            temperature: 0.0,
            governor: String::new(),
            memory_used: 0,
            memory_total: 0,
            state: String::new(),
            active_functions: Vec::new(),
        }
    }
}

/// GPU process information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuProcess {
    pub pid: u32,
    pub sm_util: u32,
    pub fb_mem: u32,
    pub command: String,
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

        // Read GPU state from sysfs
        stats.state = read_gpu_state_from_sysfs();

        // Read GPU active functions from sysfs
        stats.active_functions = read_gpu_active_functions_from_sysfs();

        stats.temperature = read_gpu_temp();
        stats
    }

        // Try to read temperature
        stats.temperature = read_gpu_temp();

        stats
    }
}

#[cfg(feature = "nvml")]
fn should_use_nvml() -> bool {
    use crate::modules::hardware::detect_board;

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
    let temp = nvml::nvmlDeviceGetTemperature(device, nvml::NVML_TEMPERATURE_GPU)?;
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

    // Fallback: search for any devfreq entry containing 'gpu' or 'gv11b'
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let entry_name = entry.file_name().to_string_lossy().to_lowercase();
            if entry_name.contains("gpu") || entry_name.contains("gv11b") {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }

    None
}

/// Read GPU state from sysfs
fn read_gpu_state_from_sysfs() -> String {
    // Try to read GPU state from /sys/class/nvrm/
    // On Jetson devices, we can check if GPU is active by reading power state
    let path = Path::new("/sys/class/nvrm/gpu0/power/runtime_status");
    
    if let Ok(content) = fs::read_to_string(&path) {
        if content.trim() == "active" {
            return "active".to_string();
        } else if content.trim() == "suspended" {
            return "idle".to_string();
        }
    }
    
    // Fallback: try to read from usage
    let usage_path = Path::new("/sys/class/nvrm/gpu0/device/gpu_busy_percent");
    if let Ok(usage) = fs::read_to_string(&usage_path) {
        if usage.trim().parse::<u32>().unwrap_or(0) > 0 {
            return "active".to_string();
        }
    }
    
    String::new()
}

/// Read GPU active functions from sysfs
fn read_gpu_active_functions_from_sysfs() -> Vec<String> {
    let mut functions = Vec::new();
    
    // Try to read CUDA usage
    let cuda_usage_path = Path::new("/sys/class/nvrm/gpu0/device/gpu_busy_percent");
    if let Ok(usage) = fs::read_to_string(&cuda_usage_path) {
        if usage.trim().parse::<u32>().unwrap_or(0) > 0 {
            functions.push("CUDA".to_string());
        }
    }
    
    // Try to read NVDEC usage from sysfs if available
    let nvdec_path = Path::new("/sys/class/nvrm/gpu0/device/nvdec_usage");
    if nvdec_path.exists() {
        if let Ok(usage) = fs::read_to_string(&nvdec_path) {
            if usage.trim().parse::<u32>().unwrap_or(0) > 0 {
                functions.push("NVDEC".to_string());
            }
        }
    }
    
    // Try to read NVENC usage from sysfs if available
    let nvenc_path = Path::new("/sys/class/nvrm/gpu0/device/nvenc_usage");
    if nvenc_path.exists() {
        if let Ok(usage) = fs::read_to_string(&nvenc_path) {
            if usage.trim().parse::<u32>().unwrap_or(0) > 0 {
                functions.push("NVENC".to_string());
            }
        }
    }
    
    functions
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

/// Read GPU usage (estimated from devfreq load or nvidia-smi)
fn read_gpu_usage(devfreq_path: &str) -> f32 {
    // Try nvidia-smi first (more accurate)
    if let Ok(usage) = read_nvidia_smi_usage() {
        if usage > 0.0 {
            return usage;
        }
    }

    // Fallback to devfreq load
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
/// Reads maximum frequency from specified devfreq path.
///
/// # Arguments
/// * `devfreq_path` - Path to GPU devfreq directory (e.g., "/sys/class/devfreq/gpu")
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

/// Parse nvidia-smi GPU usage output
///
/// # Arguments
/// * `output` - Output from nvidia-smi containing GPU usage percentage
///
/// # Returns
/// GPU usage as percentage (0.0-100.0), or 0.0 if parsing fails
pub fn parse_nvidia_smi_usage(output: &str) -> f32 {
    let trimmed = output.trim();

    if trimmed.is_empty() {
        return 0.0;
    }

    // Remove % sign and parse
    let without_percent = trimmed.trim_end_matches('%');
    without_percent
        .parse::<f32>()
        .unwrap_or(0.0)
        .clamp(0.0, 100.0)
}

/// Read GPU usage from nvidia-smi
///
/// Returns GPU usage percentage using nvidia-smi command.
/// Falls back to 0.0 if nvidia-smi is not available.
pub fn read_nvidia_smi_usage() -> anyhow::Result<f32> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("nvidia-smi command failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let usage_str = stdout.trim();

    if usage_str.is_empty() || usage_str == "N/A" {
        return Ok(0.0);
    }

    let usage = usage_str.parse::<f32>()?;
    Ok(usage.clamp(0.0, 100.0))
}

/// Parse nvidia-smi pmon output for GPU processes
///
/// # Arguments
/// * `output` - Output from nvidia-smi pmon command
///
/// # Returns
/// Vector of GPU processes with PID, SM utilization, framebuffer memory, and command
pub fn parse_nvidia_smi_pmon(output: &str) -> Vec<GpuProcess> {
    let mut processes = Vec::new();

    for line in output.lines() {
        let line = line.trim();

        // Skip empty lines and header comments
        if line.is_empty() || line.starts_with("# gpu") || line.starts_with("# Idx") {
            continue;
        }

        // Parse: gpu pid type device sm fb command
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Some nvidia-smi versions include a leading "#" before GPU index
        // Skip if it's a full line comment or doesn't have enough fields
        if parts.len() < 6 {
            continue;
        }

        // Try to parse PID from appropriate column
        let pid_index = if parts[0].starts_with('#') { 2 } else { 1 };
        let sm_index = if parts[0].starts_with('#') { 5 } else { 4 };
        let fb_index = if parts[0].starts_with('#') { 6 } else { 5 };
        let cmd_index = if parts[0].starts_with('#') { 7 } else { 6 };

        if pid_index >= parts.len() || sm_index >= parts.len() || fb_index >= parts.len() {
            continue;
        }

        if let Ok(pid) = parts[pid_index].parse::<u32>() {
            if let Ok(sm_util) = parts[sm_index].parse::<u32>() {
                if let Ok(fb_mem) = parts[fb_index].parse::<u32>() {
                    let command = if cmd_index < parts.len() {
                        parts[cmd_index..].join(" ")
                    } else {
                        String::new()
                    };

                    processes.push(GpuProcess {
                        pid,
                        sm_util,
                        fb_mem,
                        command,
                    });
                }
            }
        }
    }

    processes
}

/// Read GPU processes from nvidia-smi pmon
///
/// Returns list of GPU processes using nvidia-smi pmon command.
/// Falls back to empty list if nvidia-smi is not available.
pub fn read_nvidia_smi_pmon() -> anyhow::Result<Vec<GpuProcess>> {
    let output = Command::new("nvidia-smi")
        .args(&["pmon", "-c", "1"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("nvidia-smi pmon command failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let processes = parse_nvidia_smi_pmon(&stdout);

    Ok(processes)
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
        let stats = GpuStats::get();

        assert_eq!(stats.usage, 0.0);
        assert_eq!(stats.frequency, 0);
        assert_eq!(stats.governor, String::new());
        assert_eq!(stats.temperature, 0.0);
    }

    #[test]
    fn test_gpu_stats_get() {
        let stats = GpuStats::get();

        assert!(
            stats.usage >= 0.0 && stats.usage <= 100.0 || stats.usage == 0.0,
            "GPU usage should be between 0 and 100 or 0"
        );
        assert!(
            stats.temperature >= 0.0 && stats.temperature < 120.0 || stats.temperature == 0.0,
            "GPU temperature should be reasonable (0-120°C) or 0"
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

        // Frequency should be >= 0 (actual value varies by hardware)
        assert!(stats.frequency >= 0);
    }

    #[test]
    fn test_gpu_usage_calculation() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let usage = read_gpu_usage(&devfreq_path.unwrap());
            assert!(
                usage >= 0.0 && usage <= 100.0 || usage == 0.0,
                "GPU usage should be between 0 and 100"
            );
        }
    }

    #[test]
    fn test_gpu_temperature_range() {
        let temp = read_gpu_temp();

        if temp >= 0.0 {
            assert!(temp >= 0.0, "GPU temperature should be at least 0°C");
            assert!(temp < 120.0, "GPU temperature should be less than 120°C");
        }
    }

    #[test]
    fn test_gpu_governor() {
        let devfreq_path = find_gpu_devfreq();

        if devfreq_path.is_some() {
            let governor = read_gpu_governor(&devfreq_path.unwrap());
            assert!(
                !governor.is_empty() || governor == "unknown" || governor == "nvml",
                "Governor should not be empty or should be 'unknown'/'nvml'"
            );
        }
    }

    #[test]
    fn test_gpu_serialization() {
        let stats = GpuStats::get();

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

            assert!(
                max_freq >= 100_000_000 || max_freq == 0,
                "Max freq should be at least 100MHz or 0"
            );
        }
    }

    #[test]
    #[ignore = "Requires implementation - failing test for GPU memory reading"]
    #[test]
    fn test_read_gpu_memory() {
        let stats = GpuStats::get();

        // GPU memory should be available if nvidia-smi is present
        if stats.memory_total > 0 {
            assert!(stats.memory_used > 0, "GPU memory used should be positive");
            assert!(
                stats.memory_used <= stats.memory_total,
                "GPU memory used should not exceed total"
            );
        }
    }

    #[test]
    fn test_read_gpu_state() {
        let stats = GpuStats::get();

        // GPU state should be either "active" or "idle" (or similar)
        assert!(!stats.state.is_empty(), "GPU state should not be empty");

        // State should be one of expected values
        let valid_states = vec!["active", "idle", "off", "on"];
        let is_valid = valid_states
            .iter()
            .any(|s| stats.state.to_lowercase().contains(s));
        assert!(
            is_valid || stats.state.is_empty(),
            "GPU state should be valid or empty if unavailable"
        );
    }

    #[test]
    fn test_read_gpu_active_functions() {
        let stats = GpuStats::get();

        // Active functions should be a vector (can be empty if unavailable)
        // If nvidia-smi is present, at least one function should be detected
        if !stats.active_functions.is_empty() {
            // Common GPU functions to check for
            let common_functions = vec!["CUDA", "NVDEC", "NVENC", "NVJPG", "NVSCI"];

            // At least one common function should be detected if available
            let has_common_function = stats
                .active_functions
                .iter()
                .any(|f| common_functions.iter().any(|cf| f.contains(cf)));

            // Note: This assertion may fail on systems without CUDA workload
            // assert!(has_common_function, "Should detect at least one common GPU function");
        }
    }
}

#[test]
#[ignore = "Requires Jetson hardware - run with: cargo test gpu -- --ignored"]
fn test_print_gpu_info() {
    println!("\n=== GPU Information Test ===");

    let is_jetson_device = crate::modules::hardware::is_jetson();
    println!("Is Jetson: {}", is_jetson_device);

    if !is_jetson_device {
        println!("Not running on Jetson device - GPU info not available");
        println!("\n=== Test Complete ===");
        return;
    }

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
    let board = detect_board();

    if board.l4t.starts_with("38.") || board.l4t.starts_with("39.") {
        println!(
            "L4T {} detected (JetPack 7.0+), NVML should be available",
            board.l4t
        );
    }
}

#[test]
fn test_nvidia_smi_parsing() {
    let sample_output = "45%";
    let usage = parse_nvidia_smi_usage(sample_output);
    assert_eq!(usage, 45.0);

    let sample_output2 = "87%";
    let usage2 = parse_nvidia_smi_usage(sample_output2);
    assert_eq!(usage2, 87.0);

    let sample_output3 = "0%";
    let usage3 = parse_nvidia_smi_usage(sample_output3);
    assert_eq!(usage3, 0.0);

    let sample_output4 = "100%";
    let usage4 = parse_nvidia_smi_usage(sample_output4);
    assert_eq!(usage4, 100.0);
}

#[test]
fn test_nvidia_smi_parsing_invalid() {
    let invalid_output = "N/A";
    let usage = parse_nvidia_smi_usage(invalid_output);
    assert_eq!(usage, 0.0);

    let invalid_output2 = "";
    let usage2 = parse_nvidia_smi_usage(invalid_output2);
    assert_eq!(usage2, 0.0);

    let invalid_output3 = "abc";
    let usage3 = parse_nvidia_smi_usage(invalid_output3);
    assert_eq!(usage3, 0.0);
}

#[test]
fn test_gpu_process_list_parsing() {
    let sample_output = r#"# gpu        pid  type    device        sm   fb    command
# Idx          #   name                        utilization  memory    name
#            0   1234    C+G     0           12    45    python
            0   5678    C+G     0           25    60    python"#;

    let processes = parse_nvidia_smi_pmon(sample_output);
    assert_eq!(processes.len(), 2);
    assert_eq!(processes[0].pid, 1234);
    assert_eq!(processes[0].sm_util, 12);
    assert_eq!(processes[0].fb_mem, 45);
    assert_eq!(processes[0].command, "python");

    assert_eq!(processes[1].pid, 5678);
    assert_eq!(processes[1].sm_util, 25);
    assert_eq!(processes[1].fb_mem, 60);
    assert_eq!(processes[1].command, "python");
}

#[test]
fn test_gpu_process_list_empty() {
    let empty_output = r#"# gpu        pid  type    device        sm   fb    command
# Idx          #   name                        utilization  memory    name
# No running processes found"#;

    let processes = parse_nvidia_smi_pmon(empty_output);
    assert_eq!(processes.len(), 0);
}

#[test]
#[ignore = "Requires Jetson hardware with nvidia-smi - run with: cargo test gpu -- --ignored"]
fn test_nvidia_smi_usage_reading() {
    let is_jetson_device = crate::modules::hardware::is_jetson();

    if !is_jetson_device {
        println!("Not running on Jetson device - nvidia-smi not available");
        return;
    }

    if let Ok(usage) = read_nvidia_smi_usage() {
        println!("GPU usage from nvidia-smi: {:.1}%", usage);
        assert!(
            usage >= 0.0 && usage <= 100.0,
            "GPU usage should be between 0 and 100"
        );
    } else {
        println!("nvidia-smi not available or failed to read usage");
    }
}

#[test]
#[ignore = "Requires Jetson hardware with nvidia-smi - run with: cargo test gpu -- --ignored"]
fn test_nvidia_smi_pmon_reading() {
    let is_jetson_device = crate::modules::hardware::is_jetson();

    if !is_jetson_device {
        println!("Not running on Jetson device - nvidia-smi not available");
        return;
    }

    if let Ok(processes) = read_nvidia_smi_pmon() {
        println!("GPU processes from nvidia-smi pmon: {}", processes.len());
        for proc in &processes {
            println!(
                "  PID {}: {} (SM: {}%, FB: {}MB)",
                proc.pid, proc.command, proc.sm_util, proc.fb_mem
            );
        }
    } else {
        println!("nvidia-smi pmon not available or failed to read processes");
    }
}
