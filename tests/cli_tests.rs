// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! CLI integration tests

use std::process::Command;

#[test]
fn test_cli_stats_json_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--stats"])
        .output()
        .expect("Failed to execute rjtop-cli");

    assert!(output.status.success(), "CLI should exit with success");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Output should not be empty");

    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(json.get("cpu").is_some(), "Should have CPU stats");
}

#[test]
fn test_cli_fan_speed_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--fan", "50"])
        .output()
        .expect("Failed to execute rjtop-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        assert!(
            stdout.contains("Fan speed set to 50") || stdout.contains("50%"),
            "Should confirm fan speed was set"
        );
    } else {
        assert!(
            stderr.contains("Permission denied")
                || stderr.contains("sudo")
                || stderr.contains("root"),
            "Should show permission/sudo error on non-root"
        );
    }
}

#[test]
fn test_cli_fan_speed_validation() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--fan", "101"])
        .output()
        .expect("Failed to execute rjtop-cli");

    assert!(
        !output.status.success(),
        "CLI should fail with invalid fan speed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("0-100") || stderr.contains("100"),
        "Should show error about valid fan speed range"
    );
}

#[test]
fn test_cli_nvpmodel_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--nvpmodel", "0"])
        .output()
        .expect("Failed to execute rjtop-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(
            stdout.contains("NVP model") || stdout.contains("model"),
            "Should confirm NVP model change"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("sudo") || stderr.contains("root") || stderr.contains("permission"),
            "Should mention sudo/permission requirement if failed"
        );
    }
}

#[test]
fn test_cli_nvpmodel_invalid_id() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--nvpmodel", "16"])
        .output()
        .expect("Failed to execute rjtop-cli");

    assert!(
        !output.status.success(),
        "CLI should fail with invalid NVP model ID"
    );
}

#[test]
fn test_cli_jetson_clocks_toggle() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--jetson-clocks"])
        .output()
        .expect("Failed to execute rjtop-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(
            stdout.contains("jetson_clocks") || stdout.contains("toggled"),
            "Should confirm jetson_clocks toggle"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("sudo") || stderr.contains("root") || stderr.contains("permission"),
            "Should mention sudo/permission requirement if failed"
        );
    }
}

#[test]
fn test_cli_export_otlp_endpoint() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rjtop-cli",
            "--",
            "--export",
            "otlp",
            "--endpoint",
            "http://localhost:4318",
        ])
        .output()
        .expect("Failed to execute rjtop-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        assert!(
            stdout.contains("OTLP") || stdout.contains("exported"),
            "Should confirm export to OTLP"
        );
    } else {
        assert!(
            stderr.contains("telemetry feature")
                || stderr.contains("feature")
                || stderr.contains("localhost"),
            "Should show feature error or attempt export"
        );
    }
}

#[test]
fn test_cli_no_arguments() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli"])
        .output()
        .expect("Failed to execute rjtop-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rjtop") || stdout.contains("help") || stdout.contains("usage"),
        "Should show help or usage when no arguments provided"
    );
}

#[test]
fn test_cli_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rjtop-cli", "--", "--help"])
        .output()
        .expect("Failed to execute rjtop-cli");

    assert!(output.status.success(), "CLI should exit with success");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--stats"), "Should mention --stats flag");
    assert!(stdout.contains("--export"), "Should mention --export flag");
    assert!(stdout.contains("--fan"), "Should mention --fan flag");
    assert!(
        stdout.contains("--nvpmodel"),
        "Should mention --nvpmodel flag"
    );
    assert!(
        stdout.contains("--jetson-clocks"),
        "Should mention --jetson-clocks flag"
    );
}
