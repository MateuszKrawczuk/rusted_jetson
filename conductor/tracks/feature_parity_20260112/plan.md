# Plan: Feature Parity with jetson-stats

## Phase 1: Hardware Detection Enhancement

### Tasks

- [x] Task: Write tests for hardware detection module [7fe5423]
  - [x] Write tests for board model detection from sysfs
  - [x] Write tests for L4T version parsing
  - [x] Write tests for Jetpack version detection
  - [x] Write tests for board serial number extraction
  - [x] Write tests for architecture detection
  - [x] Write tests for NVIDIA Thor (tegra264) platform identification
  - [x] Write tests for compatible device tree parsing

- [x] Task: Implement hardware detection module enhancements [c9086f6]
  - [x] Implement board model detection from /sys/firmware/devicetree/base/model
  - [x] Implement L4T version parsing from /etc/nv_tegra_release
  - [x] Implement Jetpack version detection logic
  - [x] Implement board serial number extraction
  - [x] Implement architecture detection for different SoC variants
  - [x] Add support for NVIDIA Thor (tegra264) platform identification
  - [x] Add compatible device tree parsing for Thor platform
  - [x] Add graceful fallback for unknown platforms

- [x] Task: Fix build issues for library testing [51ffe0c]
  - [x] Fixed unclosed delimiter in tui/screens/cpu_screen.rs
  - [x] Fixed duplicate imports and syntax errors
  - [x] Fixed crossterm import in tui/app.rs
  - [x] Removed TUI from default features to allow library build
  - [x] Verified library and CLI build successfully

- [x] Task: Improve hardware tests [87dd289]
  - [x] Fixed L4T version parsing tests with correct format
  - [x] Fixed smart quotes causing compilation errors
  - [x] Added test_print_hardware_info() to display device information
  - [x] Marked hardware test as #[ignore] for non-Jetson systems
  - [x] All unit tests pass (6 passed, 1 ignored)

- [x] Task: Add comprehensive L4T to Jetpack mapping [7d44669]
  - [x] Implemented derive_jetpack_from_l4t() with full version table
  - [x] Supports JetPack 2.x through 7.x
  - [x] Includes development preview (DP) versions
  - [x] Maps L4T 36.4 → Jetpack 6.1 correctly
  - [x] Maps L4T 36.4.3 → Jetpack 6.2 correctly
  - [x] Maps all Thor, Orin, Xavier, TX2, TX1, TK1 versions

## Phase 2: Core Monitoring Modules

### Tasks

- [x] Task: Write tests for CPU monitoring module [c49f89e]
  - [x] Write tests for CPU core count detection
  - [x] Write tests for per-core usage reading
  - [x] Write tests for CPU frequency detection
  - [x] Write tests for governor state reading
  - [x] Write tests for CPU model info extraction

- [x] Task: Implement CPU monitoring module [d41333b]
  - [x] Implement CPU core count detection from /proc/cpuinfo
  - [x] Implement per-core usage reading from /proc/stat
  - [x] Implement CPU frequency detection from sysfs
  - [x] Implement governor state reading
  - [x] Implement CPU model info extraction
  - [x] Add error handling for missing sysfs paths
  - [x] Ensure async I/O with tokio

- [x] Task: Write tests for GPU monitoring module [8f27610]
  - [x] Write tests for GPU usage reading from nvidia-smi [538424f]
  - [x] Write tests for GPU frequency detection
  - [x] Write tests for GPU frequency limits reading
  - [x] Write tests for GPU process tracking [538424f]
  - [x] Write tests for nvidia-smi parsing [538424f]
  - [x] Write tests for NVIDIA Thor GPU (tegra264) support

- [~] Task: Implement GPU monitoring module [cdfab36]
  - [x] Implement GPU usage reading via nvidia-smi command [538424f]
  - [x] Implement GPU frequency detection from sysfs
  - [x] Implement GPU frequency limits reading
  - [x] Implement GPU process tracking via nvidia-smi pmon [538424f]
  - [x] Add nvidia-smi output parsing [538424f]
  - [x] Add support for NVIDIA Thor GPU (tegra264) with CUDA 13.0+
  - [x] Add NVML support for JetPack 7.0+ (Thor)
  - [x] Handle nvidia-smi unavailability gracefully

- [x] Task: Implement GPU monitoring module [cdfab36]
  - [x] Implement GPU usage reading via nvidia-smi command [538424f]
  - [x] Implement GPU frequency detection from sysfs
  - [x] Implement GPU frequency limits reading
  - [x] Implement GPU process tracking via nvidia-smi pmon [538424f]
  - [x] Add nvidia-smi output parsing [538424f]
  - [x] Add support for NVIDIA Thor GPU (tegra264) with CUDA 13.0+
  - [x] Add NVML support for JetPack 7.0+ (Thor)
  - [x] Handle nvidia-smi unavailability gracefully

- [x] Task: Write tests for memory monitoring module [87a2142]
  - [x] Write tests for RAM usage reading [87a2142]
  - [x] Write tests for swap usage reading [87a2142]
  - [x] Write tests for EMC frequency detection [87a2142]
  - [x] Write tests for IRAM usage detection [87a2142]
  - [x] Write tests for /proc/meminfo parsing [87a2142]

- [x] Task: Implement memory monitoring module [87a2142]
  - [x] Implement RAM usage reading from /proc/meminfo [87a2142]
  - [x] Implement swap usage reading from /proc/swaps [87a2142]
  - [x] Implement EMC frequency detection from sysfs [87a2142]
  - [x] Implement IRAM usage detection [87a2142]
  - [x] Add /proc/meminfo parsing logic [87a2142]
  - [x] Ensure memory usage calculations are accurate [87a2142]

- [x] Task: Write tests for fan monitoring module [1192736]
  - [x] Write tests for fan speed reading
  - [x] Write tests for fan temperature correlation
  - [x] Write tests for fan control (speed setting)
  - [x] Write tests for auto/manual mode toggling
  - [x] Write tests for cooling device sysfs reading

- [x] Task: Implement fan monitoring module [8d78a0e]
  - [x] Implement fan speed reading from sysfs cooling devices
  - [x] Implement fan temperature correlation
  - [x] Implement fan speed control via sysfs
  - [x] Implement auto/manual mode toggling
  - [x] Add error handling for fan control permissions
  - [x] Ensure fan control works on all supported platforms

- [ ] Task: Write tests for temperature monitoring module
  - [ ] Write tests for thermal zone detection
  - [ ] Write tests for thermal zone type reading
  - [ ] Write tests for temperature value reading
  - [ ] Write tests for trip point reading
  - [ ] Write tests for thermal zone sysfs parsing

- [ ] Task: Implement temperature monitoring module
  - [ ] Implement thermal zone detection from /sys/class/thermal
  - [ ] Implement thermal zone type reading
  - [ ] Implement temperature value reading
  - [ ] Implement trip point reading
  - [ ] Add thermal zone sysfs parsing
  - [ ] Handle unavailable thermal zones gracefully

- [ ] Task: Write tests for power monitoring module
  - [ ] Write tests for INA3221 power sensor detection
  - [ ] Write tests for power rail voltage reading
  - [ ] Write tests for power rail current reading
  - [ ] Write tests for power calculation (voltage * current)
  - [ ] Write tests for hwmon sysfs parsing

- [ ] Task: Implement power monitoring module
  - [ ] Implement INA3221 power sensor detection from /sys/bus/i2c
  - [ ] Implement power rail voltage reading
  - [ ] Implement power rail current reading
  - [ ] Implement power calculation (voltage * current)
  - [ ] Add hwmon sysfs parsing
  - [ ] Handle missing power sensors gracefully

- [ ] Task: Write tests for engine monitoring module
  - [ ] Write tests for APE engine status reading
  - [ ] Write tests for DLA engine status reading
  - [ ] Write tests for NVDEC engine status reading
  - [ ] Write tests for NVENC engine status reading
  - [ ] Write tests for engine clock detection

- [ ] Task: Implement engine monitoring module
  - [ ] Implement APE engine status reading
  - [ ] Implement DLA engine status reading
  - [ ] Implement NVDEC engine status reading
  - [ ] Implement NVENC engine status reading
  - [ ] Implement engine clock detection
  - [ ] Handle unavailable engines gracefully

- [ ] Task: Write tests for process monitoring module
  - [ ] Write tests for GPU process detection
  - [ ] Write tests for process memory usage tracking
  - [ ] Write tests for nvidia-smi pmon parsing
  - [ ] Write tests for /proc/*/fd/ GPU device file checking

- [ ] Task: Implement process monitoring module
  - [ ] Implement GPU process detection via nvidia-smi pmon
  - [ ] Implement process memory usage tracking
  - [ ] Add nvidia-smi pmon output parsing
  - [ ] Add /proc/*/fd/ GPU device file checking
  - [ ] Handle process monitoring errors gracefully

- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Monitoring Modules' (Protocol in workflow.md)

## Phase 3: Control Functionality

### Tasks

- [ ] Task: Write tests for NVP model control
  - [ ] Write tests for NVP model ID reading
  - [ ] Write tests for NVP model list retrieval
  - [ ] Write tests for NVP model setting
  - [ ] Write tests for /etc/nvpmodel.conf parsing
  - [ ] Write tests for nvpmodel command execution

- [ ] Task: Implement NVP model control
  - [ ] Implement current NVP model ID reading
  - [ ] Implement available NVP model list retrieval
  - [ ] Implement NVP model setting via nvpmodel command
  - [ ] Add /etc/nvpmodel.conf parsing
  - [ ] Add error handling for nvpmodel command failures
  - [ ] Ensure NVP model control works on all platforms

- [ ] Task: Write tests for jetson_clocks control
  - [ ] Write tests for jetson_clocks state reading
  - [ ] Write tests for jetson_clocks toggling
  - [ ] Write tests for jetson_clocks command execution

- [ ] Task: Implement jetson_clocks control
  - [ ] Implement current jetson_clocks state reading
  - [ ] Implement jetson_clocks toggling via command
  - [ ] Add error handling for jetson_clocks command failures
  - [ ] Ensure jetson_clocks works on supported platforms

- [ ] Task: Conductor - User Manual Verification 'Phase 3: Control Functionality' (Protocol in workflow.md)

## Phase 4: TUI Implementation

### Tasks

- [ ] Task: Write tests for TUI app structure
  - [ ] Write tests for TUI app initialization
  - [ ] Write tests for screen state management
  - [ ] Write tests for keyboard event handling
  - [ ] Write tests for screen transitions
  - [ ] Write tests for terminal cleanup on exit

- [ ] Task: Implement TUI app structure
  - [ ] Implement TUI app initialization with ratatui
  - [ ] Implement screen state management
  - [ ] Implement keyboard event handling with crossterm
  - [ ] Implement screen transitions
  - [ ] Implement terminal cleanup on exit
  - [ ] Add error handling for terminal failures

- [ ] Task: Write tests for TUI widgets
  - [ ] Write tests for CPU widget rendering
  - [ ] Write tests for GPU widget rendering
  - [ ] Write tests for memory widget rendering
  - [ ] Write tests for fan widget rendering
  - [ ] Write tests for temperature widget rendering
  - [ ] Write tests for power widget rendering
  - [ ] Write tests for color coding consistency

- [ ] Task: Implement TUI widgets
  - [ ] Implement CPU widget with per-core usage bars
  - [ ] Implement GPU widget with usage gauge
  - [ ] Implement memory widget with multi-bar display
  - [ ] Implement fan widget with speed dial
  - [ ] Implement temperature widget with thermal zones table
  - [ ] Implement power widget with power rails list
  - [ ] Apply color coding per product guidelines (high contrast, metric-specific colors)
  - [ ] Ensure widgets update every 100ms

- [ ] Task: Write tests for TUI screens
  - [ ] Write tests for main dashboard screen
  - [ ] Write tests for CPU detail screen
  - [ ] Write tests for GPU detail screen
  - [ ] Write tests for memory detail screen
  - [ ] Write tests for temperature detail screen
  - [ ] Write tests for power detail screen
  - [ ] Write tests for control screen
  - [ ] Write tests for hardware info screen

- [ ] Task: Implement TUI screens
  - [ ] Implement main dashboard screen with all metrics overview
  - [ ] Implement CPU detail screen with per-core statistics
  - [ ] Implement GPU detail screen with frequency limits
  - [ ] Implement memory detail screen with breakdown
  - [ ] Implement temperature detail screen with thermal zones
  - [ ] Implement power detail screen with power rails
  - [ ] Implement control screen for NVP model, fan, jetson_clocks
  - [ ] Implement hardware info screen with board details
  - [ ] Add navigation hints and help text

- [ ] Task: Conductor - User Manual Verification 'Phase 4: TUI Implementation' (Protocol in workflow.md)

## Phase 5: CLI Implementation

### Tasks

- [ ] Task: Write tests for CLI argument parsing
  - [ ] Write tests for --stats flag parsing
  - [ ] Write tests for --export otlp flag parsing
  - [ ] Write tests for --fan speed command parsing
  - [ ] Write tests for --nvpmodel command parsing
  - [ ] Write tests for --jetson-clocks flag parsing
  - [ ] Write tests for --endpoint parameter parsing

- [ ] Task: Implement CLI argument parsing
  - [ ] Implement --stats flag for JSON output
  - [ ] Implement --export otlp flag with endpoint parameter
  - [ ] Implement --fan speed command
  - [ ] Implement --nvpmodel command with ID parameter
  - [ ] Implement --jetson-clocks flag
  - [ ] Add help text and usage examples

- [ ] Task: Write tests for CLI commands
  - [ ] Write tests for JSON stats output
  - [ ] Write tests for OTLP export functionality
  - [ ] Write tests for fan speed control command
  - [ ] Write tests for NVP model setting command
  - [ ] Write tests for jetson_clocks toggling command

- [ ] Task: Implement CLI commands
  - [ ] Implement JSON stats output using serde_json
  - [ ] Implement OTLP export via opentelemetry-otlp
  - [ ] Implement fan speed control command
  - [ ] Implement NVP model setting command
  - [ ] Implement jetson_clocks toggling command
  - [ ] Add error handling and user-friendly messages

- [ ] Task: Conductor - User Manual Verification 'Phase 5: CLI Implementation' (Protocol in workflow.md)

## Phase 6: Testing & Validation

### Tasks

- [ ] Task: Create comprehensive unit tests
  - [ ] Review all modules for test coverage gaps
  - [ ] Write additional unit tests to achieve >80% coverage
  - [ ] Add edge case tests for all functions
  - [ ] Add error path tests for all modules
  - [ ] Ensure all public APIs have tests

- [ ] Task: Create integration tests
  - [ ] Write integration tests for complete monitoring workflows
  - [ ] Write integration tests for TUI application lifecycle
  - [ ] Write integration tests for CLI commands
  - [ ] Write integration tests for library API usage
  - [ ] Write integration tests for OpenTelemetry export

- [ ] Task: Benchmark performance
  - [ ] Create benchmarks for TUI update latency
  - [ ] Create benchmarks for memory usage
  - [ ] Create benchmarks for monitoring operations
  - [ ] Verify <100ms update latency target
  - [ ] Verify <50MB memory footprint target

- [ ] Task: Run quality checks
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt and ensure consistent formatting
  - [ ] Run cargo test and ensure all tests pass
  - [ ] Run cargo doc and ensure documentation builds
  - [ ] Verify no security vulnerabilities with cargo-audit

- [ ] Task: Conductor - User Manual Verification 'Phase 6: Testing & Validation' (Protocol in workflow.md)

## Phase 7: Documentation

### Tasks

- [ ] Task: Complete API documentation
  - [ ] Add doc comments to all public functions
  - [ ] Add code examples to all public APIs
  - [ ] Add type information to all structs and enums
  - [ ] Run cargo doc to verify documentation builds
  - [ ] Ensure all docs follow rustdoc conventions

- [ ] Task: Create getting started guide
  - [ ] Write quick start guide for common use cases
  - [ ] Document installation methods (cargo, deb, Docker)
  - [ ] Document first run configuration
  - [ ] Document basic TUI navigation
  - [ ] Document basic CLI usage

- [ ] Task: Create troubleshooting guide
  - [ ] Document common errors and solutions
  - [ ] Document platform-specific issues
  - [ ] Document permission requirements for control operations
  - [ ] Document nvidia-smi troubleshooting
  - [ ] Document NVIDIA Thor-specific issues

- [ ] Task: Update README and project documentation
  - [ ] Update README with current features
  - [ ] Update PROJECT_OVERVIEW.md with completed functionality
  - [ ] Update ARCHITECTURE.md with current implementation
  - [ ] Add contribution guidelines link
  - [ ] Add documentation site link

- [ ] Task: Conductor - User Manual Verification 'Phase 7: Documentation' (Protocol in workflow.md)
