// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! rusted-jetsons CLI - rjtop-cli (no TUI)

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "rjtop-cli",
    author = "Mateusz Krawczuk <m.krawczuk@cybrixsystems.com>",
    version = "0.1.0",
    about = "Fast Rust-based monitoring and control for NVIDIA Jetson devices",
    long_about = "rjtop-cli provides command-line interface for monitoring and controlling NVIDIA Jetson devices. Supports JSON export, OTLP export, fan control, NVP model switching, and jetson_clocks toggling.",
    after_help = "EXAMPLES:
  rjtop-cli --stats                    Display system statistics in JSON format
  rjtop-cli --fan 75                   Set fan speed to 75%
  rjtop-cli --nvpmodel 0               Set NVP model to ID 0
  rjtop-cli --jetson-clocks            Toggle jetson_clocks
  rjtop-cli --export otlp              Export stats to OTLP endpoint
  rjtop-cli --export otlp --endpoint http://localhost:4318  Export to specific OTLP endpoint"
)]
struct Cli {
    #[arg(
        long,
        short = 's',
        help = "Display system statistics in JSON format",
        long_help = "Output comprehensive system statistics in JSON format including CPU, GPU, memory, temperature, fan, and power metrics."
    )]
    stats: bool,

    #[arg(
        long,
        value_name = "TYPE",
        help = "Export statistics to external system",
        long_help = "Export statistics to external monitoring systems. Currently supports 'otlp' for OpenTelemetry export.",
        value_parser = parse_export_type
    )]
    export: Option<String>,

    #[arg(
        long,
        value_name = "SPEED",
        help = "Set fan speed (0-100)",
        long_help = "Set fan speed manually. Requires root/sudo privileges. Values: 0 (off) to 100 (maximum). Example: --fan 75"
    )]
    fan: Option<u8>,

    #[arg(
        long,
        value_name = "ID",
        help = "Set NVP model by ID (0-15)",
        long_help = "Set NVIDIA performance model (NVP model) by ID. Requires root/sudo privileges. Use 'nvpmodel -q' to see available models."
    )]
    nvpmodel: Option<u8>,

    #[arg(
        long,
        help = "Toggle jetson_clocks",
        long_help = "Toggle jetson_clocks performance mode. Requires root/sudo privileges. Switches between maximum performance and default power modes."
    )]
    jetson_clocks: bool,

    #[arg(
        long,
        value_name = "URL",
        help = "OTLP endpoint URL for export",
        long_help = "Specify the OTLP (OpenTelemetry Protocol) endpoint URL for exporting metrics. Default: http://localhost:4318. Example: --endpoint http://localhost:4318"
    )]
    endpoint: Option<String>,
}

fn parse_export_type(s: &str) -> Result<String, String> {
    let s_lower = s.to_lowercase();
    if s_lower == "otlp" {
        Ok(s_lower)
    } else {
        Err(format!(
            "Invalid export type '{}'. Supported types: otlp",
            s
        ))
    }
}

#[derive(serde::Serialize)]
struct SystemStats {
    cpu: rusted_jetsons::CpuStats,
    gpu: rusted_jetsons::GpuStats,
    memory: rusted_jetsons::MemoryStats,
    temperature: rusted_jetsons::TemperatureStats,
    fan: rusted_jetsons::FanStats,
    power: rusted_jetsons::PowerStats,
    hardware: rusted_jetsons::BoardInfo,
}

impl SystemStats {
    fn new() -> Self {
        Self {
            cpu: rusted_jetsons::CpuStats::get(),
            gpu: rusted_jetsons::GpuStats::get(),
            memory: rusted_jetsons::MemoryStats::get(),
            temperature: rusted_jetsons::TemperatureStats::get(),
            fan: rusted_jetsons::FanStats::get(),
            power: rusted_jetsons::PowerStats::get(),
            hardware: rusted_jetsons::detect_board(),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stats {
        let stats = SystemStats::new();
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    if let Some(speed) = cli.fan {
        match rusted_jetsons::FanStats::set_speed(speed) {
            Ok(()) => {
                println!("Fan speed set to {}%", speed);
            }
            Err(e) => {
                eprintln!("Error setting fan speed: {}", e);
                eprintln!("Note: This operation requires root/sudo privileges.");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(model_id) = cli.nvpmodel {
        match rusted_jetsons::NVPModelStats::set_model(model_id) {
            Ok(()) => {
                println!("NVP model set to ID {}", model_id);
            }
            Err(e) => {
                eprintln!("Error setting NVP model: {}", e);
                eprintln!("Note: This operation requires root/sudo privileges.");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if cli.jetson_clocks {
        match rusted_jetsons::JetsonClocksStats::toggle() {
            Ok(()) => {
                println!("jetson_clocks toggled successfully");
            }
            Err(e) => {
                eprintln!("Error toggling jetson_clocks: {}", e);
                eprintln!("Note: This operation requires root/sudo privileges.");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(export_type) = cli.export {
        if export_type == "otlp" {
            let endpoint = cli
                .endpoint
                .unwrap_or_else(|| "http://localhost:4318".to_string());
            println!("Exporting to OTLP endpoint: {}", endpoint);

            #[cfg(feature = "telemetry")]
            {
                let stats = SystemStats::new();
                let exporter = rusted_jetsons::TelemetryExporter::new(endpoint);

                tokio::runtime::Runtime::new()?.block_on(async {
                    match exporter.export(&stats).await {
                        Ok(()) => println!("Successfully exported to OTLP endpoint"),
                        Err(e) => {
                            eprintln!("Error exporting to OTLP: {}", e);
                            std::process::exit(1);
                        }
                    }
                });
            }

            #[cfg(not(feature = "telemetry"))]
            {
                eprintln!("Error: OTLP export requires 'telemetry' feature to be enabled.");
                eprintln!("Rebuild with: cargo build --features telemetry");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    println!("rjtop CLI - Fast Rust-based monitoring for NVIDIA Jetson devices");
    println!("\nUsage: rjtop-cli [OPTIONS]");
    println!("\nRun 'rjtop-cli --help' for more information.");

    Ok(())
}
