# Getting Started

## Prerequisites

### System Requirements
- **Rust**: 1.70 or later
- **Operating System**: Linux (Ubuntu 20.04, 22.04, 24.04)
- **Hardware**: NVIDIA Jetson device (recommended for testing)
- **Optional**: Docker for containerized deployment

### Installing Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

## Clone and Setup

### Clone the Repository

```bash
git clone https://github.com/mkrawczuk/rusted-jetsons.git
cd rusted-jetsons
```

### Verify Repository Structure

```bash
ls -la
# Should show: Cargo.toml, src/, tui/, tests/, README.md, etc.
```

## Build the Project

### Debug Build

```bash
cargo build
```

This builds the project with debugging information enabled, useful for development.

### Release Build (Optimized)

```bash
cargo build --release
```

This builds optimized binaries with all features enabled. Use this for production deployment.

### Build with Specific Features

```bash
# TUI only (default)
cargo build --features tui

# Telemetry only
cargo build --features telemetry

# Full feature set
cargo build --features full
```

### Feature Flags

- `tui`: Terminal User Interface (ratatui + crossterm)
- `telemetry`: OpenTelemetry exports (opentelemetry_sdk + opentelemetry-otlp)
- `full`: All features enabled

## Run Applications

### TUI Application (rjtop)

```bash
# Run debug build
cargo run --bin rjtop

# Run release build
./target/release/rjtop
```

**Controls:**
- `q`: Quit
- `1-9`: Switch between screens
- `Arrow keys`: Navigate
- `Enter`: Select

### CLI Application (rjtop-cli)

```bash
# Show all stats as JSON
cargo run --bin rjtop-cli -- --stats

# Export to OTLP endpoint
cargo run --bin rjtop-cli -- --export otlp --endpoint http://localhost:4318

# Control fan speed (percentage)
cargo run --bin rjtop-cli -- --fan speed 50

# Set NVP model (0-15)
cargo run --bin rjtop-cli -- --nvpmodel 0

# Toggle jetson_clocks (requires root)
sudo cargo run --bin rjtop-cli -- --jetson-clocks
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_hardware_detection
```

### Run Integration Tests

```bash
cargo test --test integration
```

## Development Workflow

### 1. Make Changes

Edit the relevant source files:
- Core library: `src/lib.rs`, `src/error.rs`
- Monitoring modules: `src/modules/*.rs`
- TUI: `tui/*.rs`
- CLI: `src/main.rs`, `src/main_cli.rs`

### 2. Build and Check

```bash
# Check for compilation errors
cargo check

# Build with warnings as errors
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run tests with code coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

### 4. Verify Functionality

```bash
# Run TUI to verify changes
cargo run --bin rjtop

# Run CLI to verify JSON output
cargo run --bin rjtop-cli -- --stats
```

### 5. Prepare for Commit

```bash
# Format all code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Build release to ensure optimization doesn't break anything
cargo build --release
```

## Common Development Commands

### Check Dependencies

```bash
# Show dependency tree
cargo tree

# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update
```

### Clean Build Artifacts

```bash
# Clean all builds
cargo clean

# Clean specific target
cargo clean --release
```

### Build Debian Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Build and install .deb
cargo deb --install

# Build only
cargo deb --no-build
```

## Troubleshooting

### Build Errors

**Error: `linker not found`**
```bash
# Install build essentials
sudo apt-get update
sudo apt-get install build-essential
```

**Error: `procfs not found`**
This is expected if not running on a Jetson device. The code includes mock data for development.

### TUI Rendering Issues

**Problem: Garbled output or colors**
```bash
# Check terminal capabilities
echo $TERM
export TERM=xterm-256color
```

### Permission Issues

**Problem: Can't control fan or clocks**
```bash
# Run with sudo for control operations
sudo ./target/release/rjtop-cli --jetson-clocks
```

## IDE Setup

### VSCode

Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.

### IntelliJ IDEA

Install the Rust plugin from JetBrains Marketplace.

## Testing on External Devices

### Configuration

Create a `.env` file in the project root (copy from `.env.example`):

```bash
cp .env.example .env
```

### Available Test Devices

Details about test devices (IP addresses, logins, and access restrictions) are stored in `.env` file.

**Note**: The `.env` file contains sensitive credentials and is excluded from git via `.gitignore`. Never commit this file to the repository.

**Note**: The `.env` file contains sensitive credentials and is excluded from git via `.gitignore`. Never commit this file to the repository.

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for code structure
- Explore [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) for features
- Check the original [jetson-stats](https://github.com/rbonghi/jetson_stats) for reference
