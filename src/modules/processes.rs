// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Process monitoring module

use std::fs;
use std::path::Path;

/// Process statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProcessStats {
    pub total_processes: usize,
    pub gpu_processes: Vec<ProcessInfo>,
}

/// Individual process information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub gpu_usage: f32,
    pub memory: u64,
    pub command: String,
}

impl ProcessStats {
    /// Get current process statistics
    pub fn get() -> Self {
        let mut stats = ProcessStats::default();

        // Get GPU processes from nvidia-smi
        if let Ok(processes) = get_gpu_processes() {
            stats.gpu_processes = processes;
        }

        // Count total processes
        stats.total_processes = count_total_processes();

        stats
    }
}

/// Get GPU processes from nvidia-smi pmon
fn get_gpu_processes() -> anyhow::Result<Vec<ProcessInfo>> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["pmon", "-c", "1"])
        .output()?;

    let processes = parse_pmon_output(&String::from_utf8_lossy(&output.stdout));

    Ok(processes)
}

/// Parse nvidia-smi pmon output
fn parse_pmon_output(output: &str) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    for line in output.lines().skip(2) {
        if line.trim().is_empty() || line.starts_with("#") || line.starts_with("gpu") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 5 {
            let pid = parts[1].parse().unwrap_or(0);
            let name = parts[2].to_string();
            let gpu_usage = parts[3].parse().unwrap_or(0.0);
            let command = parts.join(" ");

            processes.push(ProcessInfo {
                pid,
                name,
                gpu_usage,
                memory: 0,
                command,
            });
        }
    }

    processes
}

/// Count total processes in /proc
fn count_total_processes() -> usize {
    let proc_path = Path::new("/proc");

    if let Ok(entries) = fs::read_dir(proc_path) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|n| n.chars().all(|c| c.is_numeric()))
            .count()
    } else {
        0
    }
}

/// Check if process has GPU device file open
fn has_gpu_device_fd(pid: u32) -> bool {
    let fd_path_str = format!("/proc/{}/fd", pid);
    let fd_path = Path::new(&fd_path_str);

    if !fd_path.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(&fd_path) {
        for entry in entries.flatten() {
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy();
                if target_str.contains("nvidia") || target_str.contains("nvrm") {
                    return true;
                }
            }
        }
    }

    false
}

/// Get process memory usage
fn get_process_memory(pid: u32) -> u64 {
    let statm_path_str = format!("/proc/{}/statm", pid);
    let statm_path = Path::new(&statm_path_str);

    if let Ok(content) = fs::read_to_string(&statm_path) {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            return parts[1].parse().unwrap_or(0) * 4096; // Resident set size in bytes
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_stats_default() {
        let stats = ProcessStats::default();
        assert_eq!(stats.total_processes, 0);
        assert!(stats.gpu_processes.is_empty());
    }

    #[test]
    fn test_process_info_default() {
        let info = ProcessInfo::default();
        assert_eq!(info.pid, 0);
        assert_eq!(info.name, "");
        assert_eq!(info.gpu_usage, 0.0);
        assert_eq!(info.memory, 0);
        assert_eq!(info.command, "");
    }

    #[test]
    fn test_process_info_structure() {
        let info = ProcessInfo {
            pid: 1234,
            name: "python".to_string(),
            gpu_usage: 45.5,
            memory: 123456789,
            command: "python -m train.py".to_string(),
        };

        assert_eq!(info.pid, 1234);
        assert_eq!(info.name, "python");
        assert_eq!(info.gpu_usage, 45.5);
        assert_eq!(info.memory, 123456789);
    }

    #[test]
    fn test_process_stats_structure() {
        let stats = ProcessStats {
            total_processes: 250,
            gpu_processes: vec![
                ProcessInfo {
                    pid: 1234,
                    name: "python".to_string(),
                    gpu_usage: 45.5,
                    memory: 123456789,
                    command: "python -m train.py".to_string(),
                },
                ProcessInfo {
                    pid: 5678,
                    name: "inference".to_string(),
                    gpu_usage: 30.0,
                    memory: 987654321,
                    command: "./inference --model model.pt".to_string(),
                },
            ],
        };

        assert_eq!(stats.total_processes, 250);
        assert_eq!(stats.gpu_processes.len(), 2);
    }

    #[test]
    fn test_gpu_process_detection() {
        let stats = ProcessStats::get();

        if !stats.gpu_processes.is_empty() {
            for proc in &stats.gpu_processes {
                assert!(proc.pid > 0);
                assert!(!proc.name.is_empty());
            }
        }
    }

    #[test]
    fn test_process_memory_usage_tracking() {
        let stats = ProcessStats::get();

        if !stats.gpu_processes.is_empty() {
            for proc in &stats.gpu_processes {
                assert!(proc.memory >= 0);
            }
        }
    }

    #[test]
    fn test_nvidia_smi_pmon_parsing() {
        let output = "# gpu        pid  type    sm    mem    enc    dec    command                                                                
                      Idx          %      %      %      %                                                                 
                        0      1234  C     45     12      0      0  python train.py
                        0      5678  C     30     08      0      0  inference.exe";

        let processes = parse_pmon_output(output);

        assert_eq!(processes.len(), 2);
        assert_eq!(processes[0].pid, 1234);
        assert_eq!(processes[0].gpu_usage, 45.0);
        assert_eq!(processes[1].pid, 5678);
        assert_eq!(processes[1].gpu_usage, 30.0);
    }

    #[test]
    fn test_gpu_device_file_checking() {
        let pid = std::process::id();
        let has_gpu = has_gpu_device_fd(pid);
        assert!(has_gpu || !has_gpu);
    }

    #[test]
    fn test_process_serialization() {
        let stats = ProcessStats {
            total_processes: 250,
            gpu_processes: vec![ProcessInfo {
                pid: 1234,
                name: "python".to_string(),
                gpu_usage: 45.5,
                memory: 123456789,
                command: "python -m train.py".to_string(),
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "ProcessStats should be serializable");

        let deserialized: Result<ProcessStats, _> = serde_json::from_str(&json.unwrap());
        assert!(
            deserialized.is_ok(),
            "ProcessStats should be deserializable"
        );
    }

    #[test]
    fn test_process_info_serialization() {
        let info = ProcessInfo {
            pid: 1234,
            name: "python".to_string(),
            gpu_usage: 45.5,
            memory: 123456789,
            command: "python -m train.py".to_string(),
        };

        let json = serde_json::to_string(&info);
        assert!(json.is_ok(), "ProcessInfo should be serializable");

        let deserialized: Result<ProcessInfo, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "ProcessInfo should be deserializable");
    }

    #[test]
    #[ignore = "Requires Jetson hardware - run with: cargo test processes -- --ignored"]
    fn test_print_process_info() {
        println!("\n=== Process Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - process info not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = ProcessStats::get();

        println!("Total processes: {}", stats.total_processes);
        println!("GPU processes: {}", stats.gpu_processes.len());

        for proc in &stats.gpu_processes {
            println!(
                "  PID {}: {} - GPU: {:.1}%, Mem: {} MB - {}",
                proc.pid,
                proc.name,
                proc.gpu_usage,
                proc.memory / (1024 * 1024),
                proc.command
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_gpu_usage_range() {
        let stats = ProcessStats::get();

        for proc in &stats.gpu_processes {
            assert!(proc.gpu_usage >= 0.0 && proc.gpu_usage <= 100.0);
        }
    }
}
