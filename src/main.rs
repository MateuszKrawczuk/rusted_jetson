// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! rusted-jetsons CLI - rjtop

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    stats: bool,

    #[arg(long)]
    export: Option<String>,

    #[arg(long, value_name = "SPEED")]
    fan: Option<u8>,

    #[arg(long, value_name = "ID")]
    nvpmodel: Option<u8>,

    #[arg(long)]
    jetson_clocks: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.stats {
        println!("{{\"cpu\": {{\"usage\": 50.0}}}}");
        return Ok(());
    }

    println!("rjtop TUI starting...");
    Ok(())
}
