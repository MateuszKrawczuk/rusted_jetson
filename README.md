# rusted-jetsons

Fast Rust-based monitoring and control for NVIDIA Jetson devices.

SPDX-License-Identifier: LGPL-3.0
Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

## Original Project

This is a Rust fork of [jetson-stats](https://github.com/rbonghi/jetson_stats) by Raffaello Bonghi (AGPL-3.0).

rusted-jetsons provides identical functionality but with:
- Faster TUI (Rust + ratatui)
- OpenTelemetry integration for Grafana/Loki
- More efficient resource usage

## Features

- **Hardware Detection**: Decode hardware, architecture, L4T and NVIDIA Jetpack
- **Monitoring**: CPU, GPU, Memory, Engines, fan, temperature, power
- **Control**: NVP model, fan speed, jetson_clocks
- **Library API**: Importable in Rust projects
- **OpenTelemetry**: Export metrics to Grafana/Loki
- **Docker Support**: Works in containers
- **No Superuser Required**: Runs without sudo for monitoring

## Installation

### From source (cargo)

```bash
cargo install --locked --features full rusted-jetsons
```

### From .deb package

```bash
wget https://github.com/mkrawczuk/rusted-jetsons/releases/download/v0.1.0/rusted-jetsons_0.1.0_amd64.deb
sudo dpkg -i rusted-jetsons_0.1.0_amd64.deb
```

### With cargo-deb

```bash
cargo install cargo-deb
cargo deb --install --no-build
```

## Usage

### CLI

```bash
# Start TUI
rjtop

# Show stats as JSON
rjtop-cli --stats

# Export to OTLP endpoint
rjtop-cli --export otlp --endpoint http://localhost:4318

# Control fan
rjtop-cli --fan speed 50

# Set NVP model
rjtop-cli --nvpmodel 0

# Toggle jetson_clocks
rjtop-cli --jetson-clocks
```

### Library API

```rust
use rusted_jetsons::{JetsonMonitor, Observer, JetsonStats};

struct MyObserver;

impl Observer for MyObserver {
    fn on_update(&self, stats: &JetsonStats) {
        println!("CPU: {}%", stats.cpu.usage);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut monitor = JetsonMonitor::new()?;
    monitor.attach_observer(Box::new(MyObserver));
    monitor.start().await?;
    Ok(())
}
```

## Supported Platforms

- NVIDIA Jetson Orin Series
- NVIDIA Jetson Xavier Series
- NVIDIA Jetson Nano
- NVIDIA Jetson TX Series
- Ubuntu 20.04 (Focal), 22.04 (Jammy), 24.04 (Noble)

## Dependencies

### CLI
- No external runtime dependencies for basic functionality

### TUI
- Requires terminal with Unicode support

### OpenTelemetry
- Network access to OTLP endpoint

## License

SPDX-License-Identifier: LGPL-3.0
See LICENSE file for details.

## Original Project

- Author: Raffaello Bonghi
- License: AGPL-3.0
- URL: https://github.com/rbonghi/jetson_stats
