// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

pub struct MockData;

impl MockData {
    pub fn new() -> Self {
        Self
    }
}

impl MockData {
    pub fn cpu_stats(&self) -> crate::cpu::CpuStats {
        use std::iter;
        crate::cpu::CpuCore;

        let mut cores = Vec::new();
        for i in 0..6 {
            cores.push(CpuCore {
                index: i,
                usage: 10.0 + i as f32,
                frequency: 1500000 + i * 100000,
                governor: "schedutil".to_string(),
            });
        }

        crate::cpu::CpuStats {
            usage: cores.iter().map(|c| c.usage).sum::<f32>() / cores.len() as f32,
            cores,
            frequency: 1500000,
        }
    }

    pub fn gpu_stats(&self) -> crate::gpu::GpuStats {
        crate::gpu::GpuStats {
            usage: 30.0,
            frequency: 1200000,
            temperature: 40.0,
            governor: "performance".to_string(),
        }
    }

    pub fn memory_stats(&self) -> crate::memory::MemoryStats {
        crate::memory::MemoryStats {
            ram_used: 4 * 1024 * 1024,
            ram_total: 8 * 1024 * 1024,
            ram_cached: 1 * 1024 * 1024,
            swap_used: 0,
            swap_total: 2 * 1024 * 1024,
            swap_cached: 0,
            iram_used: 256 * 1024,
            iram_total: 512 * 1024,
            iram_lfb: 0,
        }
    }

    pub fn temperature_stats(&self) -> crate::temperature::TemperatureStats {
        crate::temperature::TemperatureStats {
            cpu: 35.0,
            gpu: 40.0,
            board: 30.0,
            pmic: 25.0,
            thermal_zones: Vec::new(),
        }
    }

    pub fn power_stats(&self) -> crate::power::PowerStats {
        let mut rails = Vec::new();
        rails.push(crate::power::PowerRail {
            name: "CPU".to_string(),
            current: 5.0,
            voltage: 1.2,
            power: 6.0,
        });
        rails.push(crate::power::PowerRail {
            name: "GPU".to_string(),
            current: 3.0,
            voltage: 1.1,
            power: 3.3,
        });
        rails.push(crate::power::PowerRail {
            name: "SOC".to_string(),
            current: 8.0,
            voltage: 0.9,
            power: 7.2,
        });
        rails.push(crate::power::PowerRail {
            name: "DDR".to_string(),
            current: 2.0,
            voltage: 1.1,
            power: 2.2,
        });

        crate::power::PowerStats { total: 18.7, rails }
    }

    pub fn fan_stats(&self) -> crate::fan::FanStats {
        let mut fans = Vec::new();
        fans.push(crate::fan::FanInfo {
            index: 0,
            speed: 60,
            rpm: 4200,
        });

        crate::fan::FanStats {
            speed: 60,
            rpm: 4200,
            mode: crate::fan::FanMode::Auto,
            fans,
        }
    }

    pub fn board_info(&self) -> crate::hardware::BoardInfo {
        crate::hardware::BoardInfo {
            model: "Jetson Xavier NX".to_string(),
            jetpack: "5.1.2".to_string(),
            l4t: "35.3.1".to_string(),
            serial: "015000000".to_string(),
        }
    }
}
