// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Performance benchmarks for rusted-jetsons
//!
//! Run with: cargo bench
//! Or for quick validation: cargo test --test monitoring_bench --release

#![allow(unused)]

use std::time::{Duration, Instant};

use rusted_jetsons::{
    CpuStats, FanStats, GpuStats, MemoryStats, PowerStats, TemperatureStats,
    detect_board,
};

const ITERATIONS: u32 = 100;
const TARGET_LATENCY_MS: u128 = 100; // <100ms target

/// Benchmark helper - measures average duration over iterations
fn benchmark<F: FnMut()>(name: &str, iterations: u32, mut f: F) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let total = start.elapsed();
    let avg = total / iterations;
    println!("{}: avg {:?} per call ({} iterations)", name, avg, iterations);
    avg
}

#[test]
fn bench_cpu_stats_get() {
    let avg = benchmark("CpuStats::get()", ITERATIONS, || {
        let _ = CpuStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "CpuStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_memory_stats_get() {
    let avg = benchmark("MemoryStats::get()", ITERATIONS, || {
        let _ = MemoryStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "MemoryStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_temperature_stats_get() {
    let avg = benchmark("TemperatureStats::get()", ITERATIONS, || {
        let _ = TemperatureStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "TemperatureStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_power_stats_get() {
    let avg = benchmark("PowerStats::get()", ITERATIONS, || {
        let _ = PowerStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "PowerStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_gpu_stats_get() {
    let avg = benchmark("GpuStats::get()", ITERATIONS, || {
        let _ = GpuStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "GpuStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_fan_stats_get() {
    let avg = benchmark("FanStats::get()", ITERATIONS, || {
        let _ = FanStats::get();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "FanStats::get() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

#[test]
fn bench_detect_board() {
    let avg = benchmark("detect_board()", ITERATIONS, || {
        let _ = detect_board();
    });
    assert!(
        avg.as_millis() < TARGET_LATENCY_MS,
        "detect_board() should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg
    );
}

/// Benchmark complete monitoring cycle (all stats)
#[test]
fn bench_complete_monitoring_cycle() {
    let avg = benchmark("Complete monitoring cycle", ITERATIONS / 10, || {
        let _ = CpuStats::get();
        let _ = MemoryStats::get();
        let _ = TemperatureStats::get();
        let _ = PowerStats::get();
        let _ = GpuStats::get();
        let _ = FanStats::get();
    });

    // Complete cycle should still be well under TUI tick rate (250ms)
    assert!(
        avg.as_millis() < 250,
        "Complete monitoring cycle should be <250ms (TUI tick rate), was {:?}",
        avg
    );
}

/// Test that TUI update latency is acceptable
#[test]
fn bench_tui_update_latency() {
    // Simulate TUI update: read all stats and check total time
    let start = Instant::now();

    for _ in 0..10 {
        let _ = CpuStats::get();
        let _ = MemoryStats::get();
        let _ = TemperatureStats::get();
        let _ = PowerStats::get();
        let _ = GpuStats::get();
        let _ = FanStats::get();
    }

    let total = start.elapsed();
    let avg_per_update = total / 10;

    println!("TUI update latency: avg {:?} per update", avg_per_update);

    assert!(
        avg_per_update.as_millis() < TARGET_LATENCY_MS,
        "TUI update latency should be <{}ms, was {:?}",
        TARGET_LATENCY_MS,
        avg_per_update
    );
}

/// Simple memory usage estimation (process RSS after operations)
#[test]
fn bench_memory_footprint_estimate() {
    // Run operations to warm up
    for _ in 0..100 {
        let _ = CpuStats::get();
        let _ = MemoryStats::get();
    }

    // Note: Accurate memory measurement requires external tools like
    // /proc/self/statm or Valgrind. This is a simple sanity check.
    // For real memory profiling, use: cargo run --release & ps aux | grep rjtop

    println!("Memory footprint estimation requires external tools");
    println!("Target: <50MB memory footprint");
    println!("Run: cargo build --release && /usr/bin/time -v ./target/release/rjtop-cli --stats");
}
