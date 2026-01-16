// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! rusted-jetsons CLI - rjtop-cli (no TUI)

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default() {
        let args: Vec<&str> = vec![];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.stats, false);
        assert_eq!(cli.export, None);
        assert_eq!(cli.fan, None);
        assert_eq!(cli.nvpmodel, None);
        assert_eq!(cli.jetson_clocks, false);
    }
}
