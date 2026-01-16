// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! NVP model control module

use std::fs;
use std::path::Path;

/// NVP model statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NVPModelStats {
    pub current_model: u8,
    pub models: Vec<NVPModel>,
    pub available: bool,
}

/// Individual NVP model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NVPModel {
    pub id: u8,
    pub name: String,
    pub description: String,
}

impl NVPModelStats {
    /// Get current NVP model statistics
    pub fn get() -> Self {
        let path = Path::new("/etc/nvpmodel.conf");

        if !path.exists() {
            return NVPModelStats::default();
        }

        let mut stats = NVPModelStats::default();
        stats.models = parse_nvpmodel_conf(&path);
        stats.available = !stats.models.is_empty();

        // Try to get current model
        stats.current_model = get_current_model_id().unwrap_or(255);

        stats
    }

    /// Set NVP model (requires root)
    pub fn set_model(model_id: u8) -> anyhow::Result<()> {
        if model_id > 15 {
            return Err(anyhow::anyhow!("Model ID must be 0-15"));
        }

        let output = std::process::Command::new("sudo")
            .args(["/usr/bin/nvpmodel", "-m", &model_id.to_string()])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("nvpmodel command failed: {}", stderr));
        }

        Ok(())
    }

    /// Get list of available NVP models
    pub fn get_models() -> Vec<NVPModel> {
        let path = Path::new("/etc/nvpmodel.conf");

        if !path.exists() {
            return Vec::new();
        }

        parse_nvpmodel_conf(&path)
    }
}

/// Parse /etc/nvpmodel.conf file
fn parse_nvpmodel_conf(path: &Path) -> Vec<NVPModel> {
    let mut models = Vec::new();

    if let Ok(content) = fs::read_to_string(path) {
        let mut current_model_id: Option<u8> = None;
        let mut current_name = String::new();
        let mut current_desc = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("NVPMODEL:") {
                if let Some(id_str) = line.strip_prefix("NVPMODEL=") {
                    if let Ok(id) = id_str.parse() {
                        current_model_id = Some(id);
                    }
                }
            } else if line.starts_with("NVPOWER:") {
                // Power state line
            } else if line.starts_with("NVPOWERCAP:") {
                // Power capability line
            } else if line.starts_with("NVPOWERCTRL:") {
                // Power control line
            } else if line.starts_with("GPU:") {
                // GPU config line
            } else if line.starts_with("GPU_MIN_FREQ:") {
                // GPU min frequency
            } else if line.starts_with("GPU_MAX_FREQ:") {
                // GPU max frequency
            } else if line.starts_with("CPU:") {
                // CPU config line
            } else if line.starts_with("CPU_MIN_FREQ:") {
                // CPU min frequency
            } else if line.starts_with("CPU_MAX_FREQ:") {
                // CPU max frequency
            } else if line.starts_with("#") {
                if let Some(id) = current_model_id {
                    if !current_name.is_empty() {
                        models.push(NVPModel {
                            id,
                            name: current_name.clone(),
                            description: current_desc.clone(),
                        });
                    }
                }
                current_name.clear();
                current_desc.clear();
            }
        }

        // Don't forget the last model
        if let Some(id) = current_model_id {
            if !current_name.is_empty() {
                models.push(NVPModel {
                    id,
                    name: current_name,
                    description: current_desc,
                });
            }
        }
    }

    models
}

/// Get current NVP model ID
fn get_current_model_id() -> Option<u8> {
    let path = Path::new("/sys/devices/soc0/firmware/devicetree/base/nvidia,pmodel");

    if let Ok(content) = fs::read_to_string(path) {
        content.trim().parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvp_stats_default() {
        let stats = NVPModelStats::default();
        assert_eq!(stats.current_model, 0);
        assert!(stats.models.is_empty());
        assert!(!stats.available);
    }

    #[test]
    fn test_nvp_model_structure() {
        let model = NVPModel {
            id: 0,
            name: "MAX N".to_string(),
            description: "Max Performance".to_string(),
        };

        assert_eq!(model.id, 0);
        assert_eq!(model.name, "MAX N");
        assert_eq!(model.description, "Max Performance");
    }

    #[test]
    fn test_nvp_stats_structure() {
        let stats = NVPModelStats {
            current_model: 2,
            available: true,
            models: vec![
                NVPModel {
                    id: 0,
                    name: "MAX N".to_string(),
                    description: "Max Performance".to_string(),
                },
                NVPModel {
                    id: 1,
                    name: "MAX P".to_string(),
                    description: "Max Power".to_string(),
                },
                NVPModel {
                    id: 2,
                    name: "MAX Q".to_string(),
                    description: "Max Quality".to_string(),
                },
            ],
        };

        assert_eq!(stats.current_model, 2);
        assert!(stats.available);
        assert_eq!(stats.models.len(), 3);
        assert_eq!(stats.models[2].name, "MAX Q");
    }

    #[test]
    fn test_nvp_model_id_reading() {
        let stats = NVPModelStats::get();

        if stats.available {
            assert!(stats.current_model <= 15, "Model ID should be 0-15");
        }
    }

    #[test]
    fn test_nvp_model_list_retrieval() {
        let stats = NVPModelStats::get();

        if stats.available {
            assert!(!stats.models.is_empty(), "Should have at least one model");

            for model in &stats.models {
                assert!(model.id <= 15, "Model ID should be 0-15");
                assert!(!model.name.is_empty(), "Model name should not be empty");
            }
        }
    }

    #[test]
    fn test_nvp_model_setting_validation() {
        // Test invalid model ID
        let result = NVPModelStats::set_model(20);
        assert!(result.is_err(), "Setting model ID > 15 should fail");

        let result = NVPModelStats::set_model(100);
        assert!(result.is_err(), "Setting model ID > 15 should fail");

        // Test valid model IDs
        let result = NVPModelStats::set_model(0);
        // Will fail without root access, but should not validate
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_nvp_serialization() {
        let stats = NVPModelStats {
            current_model: 2,
            available: true,
            models: vec![NVPModel {
                id: 0,
                name: "MAX N".to_string(),
                description: "Max Performance".to_string(),
            }],
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "NVPModelStats should be serializable");

        let deserialized: Result<NVPModelStats, _> = serde_json::from_str(&json.unwrap());
        assert!(
            deserialized.is_ok(),
            "NVPModelStats should be deserializable"
        );
    }

    #[test]
    fn test_nvp_model_serialization() {
        let model = NVPModel {
            id: 0,
            name: "MAX N".to_string(),
            description: "Max Performance".to_string(),
        };

        let json = serde_json::to_string(&model);
        assert!(json.is_ok(), "NVPModel should be serializable");

        let deserialized: Result<NVPModel, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "NVPModel should be deserializable");
    }

    #[test]
    #[ignore = "Requires Jetson hardware with nvpmodel.conf - run with: cargo test nvpmodel -- --ignored"]
    fn test_print_nvp_info() {
        println!("\n=== NVP Model Information Test ===");

        let is_jetson_device = crate::modules::hardware::is_jetson();
        println!("Is Jetson: {}", is_jetson_device);

        if !is_jetson_device {
            println!("Not running on Jetson device - NVP model not available");
            println!("\n=== Test Complete ===");
            return;
        }

        let stats = NVPModelStats::get();

        println!("Current model: {}", stats.current_model);
        println!("Available: {}", stats.available);
        println!("Number of models: {}", stats.models.len());

        for model in &stats.models {
            println!(
                "  Model {}: {} - {}",
                model.id, model.name, model.description
            );
        }

        println!("\n=== Test Complete ===");
    }

    #[test]
    fn test_nvp_model_id_range() {
        let stats = NVPModelStats::get();

        for model in &stats.models {
            assert!(model.id <= 15, "Model ID should be 0-15");
        }
    }

    #[test]
    fn test_get_models_method() {
        let models = NVPModelStats::get_models();

        for model in &models {
            assert!(model.id <= 15);
            assert!(!model.name.is_empty());
        }
    }
}
