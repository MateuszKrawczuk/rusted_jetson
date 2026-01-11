# rusted-jetsons - Implementation Plan

## Project Overview

rusted-jetsons is a Rust-based fork of jetson-stats, a monitoring and control tool for NVIDIA Jetson devices.

**Author**: Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>
**License**: LGPL-3.0
**Original Reference**: jetson-stats by Raffaello Bonghi (AGPL-3.0), https://github.com/rbonghi/jetson_stats

## Goals

1. Provide a fast, efficient TUI for Jetson monitoring
2. Export OpenTelemetry metrics for Grafana/Loki integration
3. Support Ubuntu 20.04, 22.04, and 24.04
4. Provide both CLI and library API
5. Build .deb packages for easy installation

## Project Structure

```
rusted-jetsons/
├── Cargo.toml              # Główna konfiguracja
├── Cargo-deb.toml         # Konfiguracja dla cargo-deb
├── debian/                 # Skrypty maintainer/postinst
│   ├── postinst
│   └── prerm
├── src/
│   ├── main.rs             # CLI binary (rjtop)
│   ├── lib.rs              # Biblioteka publiczna
│   ├── error.rs            # Obsługa błędów
│   ├── telemetry.rs        # OpenTelemetry exports
│   └── modules/
│       ├── mod.rs
│       ├── hardware.rs      # Wykrywanie sprzętu
│       ├── cpu.rs          # Monitorowanie CPU
│       ├── gpu.rs          # Monitorowanie GPU
│       ├── memory.rs       # Monitorowanie pamięci
│       ├── fan.rs          # Kontrola wentylatorów
│       ├── temperature.rs   # Monitorowanie temperatur
│       ├── power.rs       # Monitorowanie zużycia energii
│       ├── nvpmodel.rs    # Kontrola NVP model
│       ├── jetson_clocks.rs # Kontrola jetson_clocks
│       ├── engine.rs       # APE, DLA, NVDEC, NVENC
│       ├── processes.rs    # Procesy GPU
│       └── tegra_stats.rs # Parsowanie tegrastats
├── tui/
│   ├── mod.rs
│   ├── app.rs            # Główna aplikacja TUI
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── cpu.rs
│   │   ├── gpu.rs
│   │   ├── memory.rs
│   │   ├── power.rs
│   │   └── control.rs
│   └── screens/
│       ├── mod.rs
│       ├── all.rs        # Główny ekran
│       ├── control.rs     # Kontrola ustawień
│       └── info.rs       # Informacje o sprzęcie
└── tests/
    └── integration/
```

## Module Dependencies

| Moduł | Źródło danych | Odpowiednik jetson-stats |
|--------|---------------|------------------------|
| `hardware.rs` | `/sys/firmware/devicetree/`, `/etc/nv_tegra_release` | `core/hardware.py` |
| `cpu.rs` | `/proc/cpuinfo`, `/proc/stat`, `/sys/devices/system/cpu/` | `core/cpu.py` |
| `gpu.rs` | `/sys/class/devfreq/`, `nvidia-smi`, `pynvml` | `core/gpu.py` |
| `memory.rs` | `/proc/meminfo`, `/proc/swaps`, `/sys/kernel/debug/clk/` | `core/memory.py` |
| `fan.rs` | `/sys/class/thermal/cooling_device*`, `/sys/kernel/debug/tegra_fan/` | `core/fan.py` |
| `temperature.rs` | `/sys/class/thermal/thermal_zone*` | `core/temperature.py` |
| `power.rs` | `/sys/bus/i2c/devices/*/iio:device*` (INA3221) | `core/power.py` |
| `nvpmodel.rs` | `sudo nvpmodel` command | `core/nvpmodel.py` |
| `jetson_clocks.rs` | `/opt/nvidia/jetson_clocks/` | `core/jetson_clocks.py` |

## Library API

```rust
// Biblioteka do importu w innych projektach
use rusted_jetsons::{JetsonMonitor, Observer};

// Observer pattern
trait Observer {
    fn on_update(&self, stats: &JetsonStats);
}

// API
struct JetsonMonitor {
    fn new() -> Result<Self>;
    fn attach_observer(&mut self, observer: Box<dyn Observer>);
    fn start(&mut self) -> Result<()>;
    fn stats(&self) -> &JetsonStats;
}

pub struct JetsonStats {
    pub cpu: CpuStats,
    pub gpu: GpuStats,
    pub memory: MemoryStats,
    pub fan: FanStats,
    pub temperature: TemperatureStats,
    pub power: PowerStats,
    pub board: BoardInfo,
}
```

## OpenTelemetry Integration

```rust
// telemetry.rs
use opentelemetry::metrics::MeterProvider;

pub struct TelemetryExporter {
    // OTLP endpoint do Grafany/Loki
    // Export metryk w standardzie OpenTelemetry
}

// Eksportowane metryki:
// - cpu_usage (gauge)
// - gpu_usage (gauge)
// - memory_used (gauge)
// - temperature (gauge)
// - power_consumption (gauge)
```

## TUI (ratatui + crossterm)

- **Ekran główny**: Podsumowanie wszystkich metryk w real-time
- **Ekran GPU**: Szczegółowe informacje GPU z wykresami
- **Ekran CPU**: Rdzenie CPU, taktowanie, governor
- **Ekran Memory**: RAM, SWAP, EMC, IRAM
- **Ekran Control**: jetson_clocks, nvpmodel, fan

## Ubuntu Support

- Rust 1.70+ (dostępny w Ubuntu 22.04, backport dla 20.04)
- Zależności:
  - `ratatui` >= 0.26
  - `crossterm` >= 0.27
  - `opentelemetry` + `opentelemetry-otlp`

## Debian Packaging

```toml
# Cargo-deb.toml
[package.deb]
name = "rusted-jetsons"
maintainer = "Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>"
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
    ["target/release/rjtop", "usr/bin/", "755"],
    ["debian/rjtop.service", "etc/systemd/system/", "644"],
]
```

## CLI

```bash
rjtop                    # Uruchom TUI
rjtop --stats             # Wyświetl statystyki (JSON)
rjtop --export otlp       # Export do OTLP endpoint
rjtop --fan speed 50      # Ustaw wentylator
rjtop --nvpmodel 0        # Ustaw NVP model
rjtop --jetson-clocks     # Włącz/jetson_clocks
```

## Implementation Phases

1. **Phase 1**: Basic project structure and modules stubs
2. **Phase 2**: Hardware detection and monitoring modules
3. **Phase 3**: TUI implementation with ratatui
4. **Phase 4**: OpenTelemetry integration
5. **Phase 5**: Debian packaging and testing
6. **Phase 6**: Documentation and examples
