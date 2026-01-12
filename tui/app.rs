// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI application structure

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::tui::screens::{
    AllScreen, ControlScreen, CpuScreen, GpuScreen, InfoScreen, MemoryScreen, PowerScreen,
    TemperatureScreen,
};
use crate::tui::state::{ScreenState, StateMessage};
use crate::JetsonStats;

/// Main TUI application
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend>,
    tx: mpsc::Sender<StateMessage>,
    rx: mpsc::Receiver<StateMessage>,
    current_screen: ScreenState,
    all_screen: AllScreen,
    control_screen: ControlScreen,
    info_screen: InfoScreen,
    cpu_screen: CpuScreen,
    gpu_screen: GpuScreen,
    memory_screen: MemoryScreen,
    power_screen: PowerScreen,
    temperature_screen: TemperatureScreen,
    stats: Option<JetsonStats>,
    should_exit: bool,
    tick_rate: Duration,
}

impl TuiApp {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        // Initialize terminal
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Enable raw mode and mouse capture
        execute!(io::stdout(), EnableMouseCapture)?;

        Ok(Self {
            terminal,
            tx,
            rx,
            current_screen: ScreenState::All,
            all_screen: AllScreen::new(),
            control_screen: ControlScreen::new(),
            info_screen: InfoScreen::new(),
            cpu_screen: CpuScreen::new(),
            gpu_screen: GpuScreen::new(),
            memory_screen: MemoryScreen::new(),
            power_screen: PowerScreen::new(),
            temperature_screen: TemperatureScreen::new(),
            stats: None,
            should_exit: false,
            tick_rate: Duration::from_millis(250),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            // Handle state messages
            while let Ok(msg) = self.rx.try_recv() {
                match msg {
                    StateMessage::SetScreen(screen) => {
                        self.current_screen = screen;
                    }
                    StateMessage::Update => {
                        // Update screens with new stats
                        if let Some(stats) = self.stats.as_ref() {
                            self.all_screen.update(stats.clone());
                        }
                        self.draw()?;
                    }
                    StateMessage::Exit => {
                        self.should_exit = true;
                    }
                    StateMessage::Error(err) => {
                        eprintln!("Error: {}", err);
                        self.should_exit = true;
                    }
                }
            }

            if self.should_exit {
                break;
            }

            // Tick
            let timeout = self
                .tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let CEvent::Key(key) = event::read()? {
                    self.handle_key(key)?;
                }
            }

            if last_tick.elapsed() >= self.tick_rate {
                self.tick();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn tick(&mut self) {
        // Collect real stats from modules
        let stats = collect_stats();
        self.stats = Some(stats.clone());

        // Update all screens with current stats
        self.all_screen.update(stats.clone());

        // Update control screen with control-specific stats
        let control_stats = crate::tui::screens::ControlStats {
            fan_speed: fan::FanStats::get().speed,
            fan_mode: "Auto".to_string(),
            jetson_clocks: false,
            jetson_clocks_status: "inactive".to_string(),
            nvpmodel_id: 0,
            nvpmodel_name: "MAXN".to_string(),
        };
        self.control_screen.update(control_stats);

        // Update info screen with hardware info
        let cpu_cores = cpu::get_core_count();
        let cpu_governor = cpu::CpuStats::get()
            .cores
            .first()
            .map(|c| c.governor.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let info_stats = crate::tui::screens::InfoStats {
            board: stats.board,
            cpu_cores,
            cpu_governor,
            gpu_name: "NVIDIA GPU".to_string(),
        };
        self.info_screen.update(info_stats);

        // Update CPU screen with detailed stats
        let full_cpu = cpu::CpuStats::get();
        let cpu_screen_stats = crate::tui::screens::CpuScreenStats {
            overall: SimpleCpuStats {
                usage: full_cpu.usage,
                frequency: full_cpu.cores.first().map(|c| c.frequency).unwrap_or(0),
            },
            cores: full_cpu
                .cores
                .into_iter()
                .map(|c| crate::tui::screens::CoreStats {
                    index: c.index,
                    usage: c.usage,
                    frequency: c.frequency,
                    governor: c.governor.clone(),
                })
                .collect(),
            fan: SimpleFanStats {
                speed: fan::FanStats::get().speed,
            },
            temperature: SimpleTemperatureStats {
                cpu: temperature::TemperatureStats::get().cpu,
                gpu: temperature::TemperatureStats::get().gpu,
            },
        };
        self.cpu_screen.update(cpu_screen_stats);

        // Update GPU screen with detailed stats
        let full_gpu = gpu::GpuStats::get();
        let gpu_screen_stats = crate::tui::screens::GpuScreenStats {
            gpu: SimpleGpuStats {
                usage: full_gpu.usage,
                frequency: full_gpu.frequency,
            },
            temperature: SimpleTemperatureStats {
                cpu: temperature::TemperatureStats::get().cpu,
                gpu: full_gpu.temperature,
            },
            gpu_name: "NVIDIA GPU".to_string(),
            gpu_arch: "Unknown".to_string(),
        };
        self.gpu_screen.update(gpu_screen_stats);

        // Update Memory screen with detailed stats
        let full_memory = memory::MemoryStats::get();
        let memory_screen_stats = crate::tui::screens::MemoryScreenStats {
            memory: SimpleMemoryStats {
                ram_used: full_memory.ram_used,
                ram_total: full_memory.ram_total,
                swap_used: full_memory.swap_used,
                swap_total: full_memory.swap_total,
            },
            full_memory,
        };
        self.memory_screen.update(memory_screen_stats);

        // Update Power screen with detailed stats
        let full_power = power::PowerStats::get();
        let power_screen_stats = crate::tui::screens::PowerScreenStats {
            power: SimplePowerStats {
                total: full_power.total,
            },
            rails: full_power
                .rails
                .into_iter()
                .map(|r| crate::tui::screens::PowerRail {
                    name: r.name.clone(),
                    current: r.current,
                    voltage: r.voltage,
                    power: r.power,
                })
                .collect(),
        };
        self.power_screen.update(power_screen_stats);

        // Update Temperature screen with detailed stats
        let full_temperature = temperature::TemperatureStats::get();
        let temp_screen_stats = crate::tui::screens::TemperatureScreenStats {
            temperature: SimpleTemperatureStats {
                cpu: full_temperature.cpu,
                gpu: full_temperature.gpu,
            },
            zones: full_temperature
                .thermal_zones
                .into_iter()
                .map(|z| crate::tui::screens::ThermalZone {
                    name: z.name.clone(),
                    current_temp: z.current_temp,
                    max_temp: z.max_temp,
                    critical_temp: z.critical_temp,
                    usage_percent: if z.critical_temp > 0.0 {
                        ((z.current_temp / z.critical_temp) * 100.0) as u16
                    } else {
                        0
                    },
                })
                .collect(),
        };
        self.temperature_screen.update(temp_screen_stats);

        let _ = self.draw();
    }

    fn collect_stats(&self) -> JetsonStats {
        // Collect stats from hardware modules
        use crate::modules::{cpu, fan, gpu, hardware, memory, power, temperature};

        JetsonStats {
            cpu: SimpleCpuStats {
                usage: cpu::CpuStats::get().usage,
                frequency: cpu::CpuStats::get()
                    .cores
                    .first()
                    .map(|c| c.frequency)
                    .unwrap_or(0),
            },
            gpu: SimpleGpuStats {
                usage: gpu::GpuStats::get().usage,
                frequency: gpu::GpuStats::get().frequency,
            },
            memory: {
                let mem = memory::MemoryStats::get();
                SimpleMemoryStats {
                    ram_used: mem.ram_used,
                    ram_total: mem.ram_total,
                    swap_used: mem.swap_used,
                    swap_total: mem.swap_total,
                }
            },
            fan: SimpleFanStats {
                speed: fan::FanStats::get().speed,
            },
            temperature: {
                let temp = temperature::TemperatureStats::get();
                SimpleTemperatureStats {
                    cpu: temp.cpu,
                    gpu: temp.gpu,
                }
            },
            power: SimplePowerStats {
                total: power::PowerStats::get().total,
            },
            board: {
                let hw = hardware::detect_board();
                SimpleBoardInfo {
                    model: hw.model,
                    jetpack: hw.jetpack,
                    l4t: hw.l4t,
                }
            },
        }
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> anyhow::Result<()> {
        use event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_exit = true;
            }
            KeyCode::Char('1') => {
                self.current_screen = ScreenState::All;
                self.draw()?;
            }
            KeyCode::Char('2') => {
                self.current_screen = ScreenState::Cpu;
                self.draw()?;
            }
            KeyCode::Char('3') => {
                self.current_screen = ScreenState::Gpu;
                self.draw()?;
            }
            KeyCode::Char('4') => {
                self.current_screen = ScreenState::Memory;
                self.draw()?;
            }
            KeyCode::Char('5') => {
                self.current_screen = ScreenState::Power;
                self.draw()?;
            }
            KeyCode::Char('6') => {
                self.current_screen = ScreenState::Temperature;
                self.draw()?;
            }
            KeyCode::Char('7') => {
                self.current_screen = ScreenState::Control;
                self.draw()?;
            }
            KeyCode::Char('8') => {
                self.current_screen = ScreenState::Info;
                self.draw()?;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| match self.current_screen {
            ScreenState::All => {
                self.all_screen.draw(f);
            }
            ScreenState::Cpu => {
                self.cpu_screen.draw(f);
            }
            ScreenState::Gpu => {
                self.gpu_screen.draw(f);
            }
            ScreenState::Memory => {
                self.memory_screen.draw(f);
            }
            ScreenState::Power => {
                self.power_screen.draw(f);
            }
            ScreenState::Temperature => {
                self.temperature_screen.draw(f);
            }
            ScreenState::Control => {
                self.control_screen.draw(f);
            }
            ScreenState::Info => {
                self.info_screen.draw(f);
            }
        })?;

        self.terminal.flush()?;
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Restore terminal
        let _ = execute!(io::stdout(), DisableMouseCapture,);
        let _ = self.terminal.show_cursor();
    }
}
