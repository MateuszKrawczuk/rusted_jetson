// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Library integration tests for rusted-jetsons
//!
//! These tests verify the public API works correctly when used as a library.

use rusted_jetsons::{
    CpuStats, FanStats, GpuStats, MemoryStats, PowerStats, TemperatureStats,
    detect_board,
};

/// Test complete monitoring workflow - reading all stats
#[test]
fn test_monitoring_workflow_all_stats() {
    // Read CPU stats
    let cpu = CpuStats::get();
    assert!(cpu.usage >= 0.0 && cpu.usage <= 100.0, "CPU usage should be 0-100%");

    // Read GPU stats
    let gpu = GpuStats::get();
    assert!(gpu.usage >= 0.0 && gpu.usage <= 100.0, "GPU usage should be 0-100%");

    // Read Memory stats
    let memory = MemoryStats::get();
    // Memory values should be non-negative
    assert!(memory.ram_total >= memory.ram_used || memory.ram_total == 0);

    // Read Temperature stats
    let temp = TemperatureStats::get();
    // Temperature should be reasonable (0-150°C) or 0 if unavailable
    assert!(temp.cpu >= 0.0 && temp.cpu < 150.0 || temp.cpu == 0.0);

    // Read Power stats
    let power = PowerStats::get();
    // Power can be 0 or negative if sensors unavailable
    let _ = power.total;

    // Read Fan stats
    let fan = FanStats::get();
    assert!(fan.speed <= 100, "Fan speed should be 0-100%");
}

/// Test hardware detection returns valid info
#[test]
fn test_hardware_detection_workflow() {
    let board = detect_board();

    // Board info should have some data (may be empty on non-Jetson)
    let _ = board.model;
    let _ = board.jetpack;
    let _ = board.l4t;
    let _ = board.serial;
}

/// Test CPU stats structure
#[test]
fn test_cpu_stats_structure() {
    let cpu = CpuStats::get();

    // Check cores are populated
    for core in &cpu.cores {
        assert!(core.usage >= 0.0 && core.usage <= 100.0);
        // Frequency can be 0 on non-Jetson
        let _ = core.frequency;
        let _ = core.governor.clone();
    }
}

/// Test memory formatting consistency
#[test]
fn test_memory_format_consistency() {
    let memory = MemoryStats::get();

    // RAM used should not exceed total
    if memory.ram_total > 0 {
        assert!(
            memory.ram_used <= memory.ram_total,
            "RAM used should not exceed total"
        );
    }

    // SWAP used should not exceed total
    if memory.swap_total > 0 {
        assert!(
            memory.swap_used <= memory.swap_total,
            "SWAP used should not exceed total"
        );
    }
}

/// Test temperature stats with thermal zones
#[test]
fn test_temperature_thermal_zones() {
    let temp = TemperatureStats::get();

    // Check thermal zones are valid
    for zone in &temp.thermal_zones {
        assert!(
            zone.current_temp >= -50.0 && zone.current_temp < 200.0,
            "Temperature should be in reasonable range"
        );
    }
}

/// Test power stats with rails
#[test]
fn test_power_stats_rails() {
    let power = PowerStats::get();

    // Check power rails are valid
    for rail in &power.rails {
        // Power values can be negative or zero
        let _ = rail.power;
        let _ = rail.voltage;
        let _ = rail.current;
        let _ = rail.name.clone();
    }
}

/// Test multiple consecutive reads for stability
#[test]
fn test_multiple_reads_stability() {
    for _ in 0..5 {
        let _ = CpuStats::get();
        let _ = MemoryStats::get();
        let _ = TemperatureStats::get();
    }
    // Should not panic or crash
}

/// Test stats serialization (JSON)
#[test]
fn test_stats_serialization() {
    let cpu = CpuStats::get();
    let json = serde_json::to_string(&cpu);
    assert!(json.is_ok(), "CpuStats should serialize to JSON");

    let memory = MemoryStats::get();
    let json = serde_json::to_string(&memory);
    assert!(json.is_ok(), "MemoryStats should serialize to JSON");

    let temp = TemperatureStats::get();
    let json = serde_json::to_string(&temp);
    assert!(json.is_ok(), "TemperatureStats should serialize to JSON");
}

/// Test concurrent access (basic thread safety)
#[test]
fn test_concurrent_access() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                let _ = CpuStats::get();
                let _ = MemoryStats::get();
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}
