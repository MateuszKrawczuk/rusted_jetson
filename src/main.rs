// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! rusted-jetsons CLI - rjtop

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    stats: bool,

    #[arg(long, value_name = "ENDPOINT")]
    export: Option<String>,

    #[arg(long, value_name = "SPEED")]
    fan: Option<u8>,

    #[arg(long, value_name = "ID")]
    nvpmodel: Option<u8>,

    #[arg(long)]
    jetson_clocks: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle stats output
    if cli.stats {
        print_json_stats()?;
        return Ok(());
    }

    // Handle export
    if let Some(endpoint) = cli.export {
        print_export_info(&endpoint)?;
        return Ok(());
    }

    // Handle fan speed control
    if let Some(speed) = cli.fan {
        control_fan(speed)?;
        return Ok(());
    }

    // Handle nvpmodel control
    if let Some(model_id) = cli.nvpmodel {
        set_nvpmodel(model_id)?;
        return Ok(());
    }

    // Handle jetson_clocks toggle
    if cli.jetson_clocks {
        toggle_jetson_clocks()?;
        return Ok(());
    }

    // Run TUI
    let mut app = rusted_jetsons::TuiApp::new()?;
    app.run()?;

    Ok(())
}

fn print_json_stats() -> anyhow::Result<()> {
    use rusted_jetsons::modules::{
        cpu, engine, fan, gpu, hardware, memory, power, processes, temperature,
    };

    let is_jetson = hardware::is_jetson();
    let board_info = hardware::detect_board();

    let cpu_stats = cpu::CpuStats::get();
    let gpu_stats = gpu::GpuStats::get();
    let memory_stats = memory::MemoryStats::get();
    let fan_stats = fan::FanStats::get();
    let temperature_stats = temperature::TemperatureStats::get();
    let power_stats = power::PowerStats::get();
    let engine_stats = engine::EngineStats::get();
    let process_stats = processes::ProcessStats::get();

    let stats = serde_json::json!({
        "hardware": {
            "model": board_info.model,
            "jetpack": board_info.jetpack,
            "l4t": board_info.l4t,
            "serial": board_info.serial,
            "is_jetson": is_jetson,
        },
        "cpu": {
            "usage": cpu_stats.usage,
            "cores": cpu::get_core_count(),
        },
        "gpu": {
            "usage": gpu_stats.usage,
            "frequency": gpu_stats.frequency,
            "temperature": gpu_stats.temperature,
            "governor": gpu_stats.governor,
        },
        "memory": {
            "ram_used": memory_stats.ram_used,
            "ram_total": memory_stats.ram_total,
            "ram_cached": memory_stats.ram_cached,
            "swap_used": memory_stats.swap_used,
            "swap_total": memory_stats.swap_total,
            "swap_cached": memory_stats.swap_cached,
            "iram_used": memory_stats.iram_used,
            "iram_total": memory_stats.iram_total,
            "iram_lfb": memory_stats.iram_lfb,
        },
        "fan": {
            "speed": fan_stats.speed,
            "rpm": fan_stats.rpm,
            "mode": fan_stats.mode.to_string(),
            "fans": fan_stats.fans,
            "temperature": fan_stats.temperature,
        },
        "temperature": {
            "cpu": temperature_stats.cpu,
            "gpu": temperature_stats.gpu,
            "board": temperature_stats.board,
            "pmic": temperature_stats.pmic,
            "thermal_zones": temperature_stats.thermal_zones,
        },
        "power": {
            "total": power_stats.total,
            "rails": power_stats.rails,
        },
        "engine": {
            "ape": engine_stats.ape,
            "dla0": engine_stats.dla0,
            "dla1": engine_stats.dla1,
            "nvdec": engine_stats.nvdec,
            "nvenc": engine_stats.nvenc,
            "nvjpg": engine_stats.nvjpg,
        },
        "processes": {
            "total_processes": process_stats.total_processes,
            "gpu_processes": process_stats.gpu_processes,
        },
    });

    println!("{}", stats);
    Ok(())
}

fn print_export_info(endpoint: &str) -> anyhow::Result<()> {
    println!("OTLP export to endpoint: {}", endpoint);
    println!("Note: OpenTelemetry export not yet implemented");
    Ok(())
}

fn control_fan(speed: u8) -> anyhow::Result<()> {
    if speed > 100 {
        anyhow::bail!("Fan speed must be between 0 and 100");
    }

    println!("Setting fan speed to {}%...", speed);

    let output = std::process::Command::new("sudo")
        .args(["/usr/bin/rjtop", "--fan", &speed.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set fan speed: {}", stderr);
    }

    println!("Fan speed set to {}%", speed);
    Ok(())
}

fn set_nvpmodel(model_id: u8) -> anyhow::Result<()> {
    println!("Setting NVP model to {}...", model_id);

    use rusted_jetsons::modules::nvpmodel;
    nvpmodel::NVPModelStats::set_model(model_id)?;

    println!("NVP model set to {}", model_id);
    Ok(())
}

fn toggle_jetson_clocks() -> anyhow::Result<()> {
    println!("Toggling jetson_clocks...");

    use rusted_jetsons::modules::jetson_clocks;
    jetson_clocks::JetsonClocksStats::toggle()?;

    println!("jetson_clocks toggled");
    Ok(())
}
