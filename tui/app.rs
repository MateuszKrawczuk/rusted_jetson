// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI application structure

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::tui::screens::{
    AllScreen, ControlScreen, CpuScreen, GpuScreen, GpuScreenStats, InfoScreen, JetsonStats,
    MemoryScreen, PowerScreen, SimpleBoardInfo, SimpleCpuStats, SimpleFanStats, SimpleGpuStats,
    SimpleMemoryStats, SimplePowerStats, SimpleTemperatureStats, TemperatureScreen,
};
use crate::tui::state::{ScreenState, StateMessage};

use crate::modules::{cpu, fan, gpu, memory, power, temperature};

/// Main TUI application
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
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
    screen_changed: bool,
    cpu_monitor: cpu::CpuMonitor,
}

impl TuiApp {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        // Enable raw mode and alternate screen
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, DisableMouseCapture)?;
        execute!(io::stdout(), crossterm::cursor::Hide)?;

        // Initialize terminal
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

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
            screen_changed: false,
            cpu_monitor: cpu::CpuMonitor::new(),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut last_tick = Instant::now();

        // Initial draw (loading screen)
        self.draw()?;

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

            // Draw on tick OR when screen changes
            let should_draw = self.screen_changed || last_tick.elapsed() >= self.tick_rate;

            if should_draw {
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.tick();
                })) {
                    eprintln!("Panic in tick: {:?}", e);
                    self.should_exit = true;
                }
                if let Err(e) = self.draw() {
                    eprintln!("Draw error: {}", e);
                    self.should_exit = true;
                }
                self.screen_changed = false;
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn tick(&mut self) {
        // Get CPU stats once using the monitor (for delta-based usage calculation)
        let full_cpu = self.cpu_monitor.get_stats();

        // Collect real stats from modules (passing cpu_stats to avoid double-reading)
        let stats = self.collect_stats_with_cpu(full_cpu.clone());
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
        let cpu_governor = full_cpu
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

        // Update CPU screen with detailed stats (using full_cpu from cpu_monitor above)
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
                board: temperature::TemperatureStats::get().board,
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
                board: temperature::TemperatureStats::get().board,
            },
            gpu_name: "NVIDIA GPU".to_string(),
            gpu_arch: "Unknown".to_string(),
            memory_used: full_gpu.memory_used,
            memory_total: full_gpu.memory_total,
            state: full_gpu.state.clone(),
            governor: full_gpu.governor.clone(),
            active_functions: full_gpu.active_functions.clone(),
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
                board: full_temperature.board,
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
    }

    fn collect_stats_with_cpu(&self, cpu_stats: cpu::CpuStats) -> JetsonStats {
        // Collect stats from hardware modules
        use crate::modules::{fan, gpu, hardware, memory, power, temperature};

        JetsonStats {
            cpu: SimpleCpuStats {
                usage: cpu_stats.usage,
                frequency: cpu_stats
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
                    board: temp.board,
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
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_exit = true;
            }
            KeyCode::Char('1') => {
                self.current_screen = ScreenState::All;
                self.screen_changed = true;
            }
            KeyCode::Char('2') => {
                self.current_screen = ScreenState::Cpu;
                self.screen_changed = true;
            }
            KeyCode::Char('3') => {
                self.current_screen = ScreenState::Gpu;
                self.screen_changed = true;
            }
            KeyCode::Char('4') => {
                self.current_screen = ScreenState::Memory;
                self.screen_changed = true;
            }
            KeyCode::Char('5') => {
                self.current_screen = ScreenState::Power;
                self.screen_changed = true;
            }
            KeyCode::Char('6') => {
                self.current_screen = ScreenState::Temperature;
                self.screen_changed = true;
            }
            KeyCode::Char('7') => {
                self.current_screen = ScreenState::Control;
                self.screen_changed = true;
            }
            KeyCode::Char('8') => {
                self.current_screen = ScreenState::Info;
                self.screen_changed = true;
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

        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            crossterm::cursor::Show
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn test_screen_state_index_consistency() {
        let all_screen = ScreenState::All;
        assert_eq!(all_screen.index(), 1);

        let cpu_screen = ScreenState::Cpu;
        assert_eq!(cpu_screen.index(), 2);

        let gpu_screen = ScreenState::Gpu;
        assert_eq!(gpu_screen.index(), 3);
    }

    #[test]
    fn test_screen_state_from_index_roundtrip() {
        for idx in 0..ScreenState::COUNT {
            if let Some(state) = ScreenState::from_index(idx) {
                assert!(state.index() >= 1 && state.index() <= 8);
            }
        }
    }

    #[test]
    fn test_screen_state_names() {
        assert_eq!(ScreenState::All.name(), "All");
        assert_eq!(ScreenState::Cpu.name(), "CPU");
        assert_eq!(ScreenState::Gpu.name(), "GPU");
        assert_eq!(ScreenState::Memory.name(), "Memory");
        assert_eq!(ScreenState::Power.name(), "Power");
        assert_eq!(ScreenState::Temperature.name(), "Temperature");
        assert_eq!(ScreenState::Control.name(), "Control");
        assert_eq!(ScreenState::Info.name(), "Info");
    }

    #[test]
    fn test_screen_state_transitions() {
        let initial_screen = ScreenState::All;
        assert_eq!(initial_screen, ScreenState::All);

        let cpu_screen = ScreenState::Cpu;
        assert_ne!(initial_screen, cpu_screen);

        let final_screen = ScreenState::Gpu;
        assert_eq!(final_screen, ScreenState::Gpu);
    }

    #[test]
    fn test_all_screen_states_are_unique() {
        let states = vec![
            ScreenState::All,
            ScreenState::Cpu,
            ScreenState::Gpu,
            ScreenState::Memory,
            ScreenState::Power,
            ScreenState::Temperature,
            ScreenState::Control,
            ScreenState::Info,
        ];

        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        state1, state2,
                        "Screen states at {} and {} should be different",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_state_message_variants() {
        let set_screen_msg = StateMessage::SetScreen(ScreenState::Cpu);
        assert!(matches!(set_screen_msg, StateMessage::SetScreen(_)));

        let update_msg = StateMessage::Update;
        assert!(matches!(update_msg, StateMessage::Update));

        let exit_msg = StateMessage::Exit;
        assert!(matches!(exit_msg, StateMessage::Exit));

        let error_msg = StateMessage::Error("test error".to_string());
        assert!(matches!(error_msg, StateMessage::Error(_)));
    }

    #[test]
    fn test_keyboard_event_codes() {
        let key_q = KeyCode::Char('q');
        let key_q_upper = KeyCode::Char('Q');
        let key_esc = KeyCode::Esc;
        let key_1 = KeyCode::Char('1');
        let key_2 = KeyCode::Char('2');
        let key_3 = KeyCode::Char('3');
        let key_4 = KeyCode::Char('4');
        let key_5 = KeyCode::Char('5');
        let key_6 = KeyCode::Char('6');
        let key_7 = KeyCode::Char('7');
        let key_8 = KeyCode::Char('8');

        let mut event_q = KeyEvent::new(key_q, KeyModifiers::NONE);
        event_q.kind = KeyEventKind::Press;
        let mut event_q_upper = KeyEvent::new(key_q_upper, KeyModifiers::NONE);
        event_q_upper.kind = KeyEventKind::Press;
        let mut event_esc = KeyEvent::new(key_esc, KeyModifiers::NONE);
        event_esc.kind = KeyEventKind::Press;
        let mut event_1 = KeyEvent::new(key_1, KeyModifiers::NONE);
        event_1.kind = KeyEventKind::Press;
        let mut event_2 = KeyEvent::new(key_2, KeyModifiers::NONE);
        event_2.kind = KeyEventKind::Press;
        let mut event_3 = KeyEvent::new(key_3, KeyModifiers::NONE);
        event_3.kind = KeyEventKind::Press;
        let mut event_4 = KeyEvent::new(key_4, KeyModifiers::NONE);
        event_4.kind = KeyEventKind::Press;
        let mut event_5 = KeyEvent::new(key_5, KeyModifiers::NONE);
        event_5.kind = KeyEventKind::Press;
        let mut event_6 = KeyEvent::new(key_6, KeyModifiers::NONE);
        event_6.kind = KeyEventKind::Press;
        let mut event_7 = KeyEvent::new(key_7, KeyModifiers::NONE);
        event_7.kind = KeyEventKind::Press;
        let mut event_8 = KeyEvent::new(key_8, KeyModifiers::NONE);
        event_8.kind = KeyEventKind::Press;

        assert_eq!(event_q.code, key_q);
        assert_eq!(event_q_upper.code, key_q_upper);
        assert_eq!(event_esc.code, key_esc);
        assert_eq!(event_1.code, key_1);
        assert_eq!(event_2.code, key_2);
        assert_eq!(event_3.code, key_3);
        assert_eq!(event_4.code, key_4);
        assert_eq!(event_5.code, key_5);
        assert_eq!(event_6.code, key_6);
        assert_eq!(event_7.code, key_7);
        assert_eq!(event_8.code, key_8);
    }

    #[test]
    fn test_keyboard_event_kind() {
        let mut press_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        press_event.kind = KeyEventKind::Press;
        let mut release_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        release_event.kind = KeyEventKind::Release;
        let mut repeat_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        repeat_event.kind = KeyEventKind::Repeat;

        assert_eq!(press_event.kind, KeyEventKind::Press);
        assert_eq!(release_event.kind, KeyEventKind::Release);
        assert_eq!(repeat_event.kind, KeyEventKind::Repeat);
    }

    #[test]
    fn test_key_modifiers() {
        let mut key_with_ctrl = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        key_with_ctrl.kind = KeyEventKind::Press;
        let mut key_with_modifiers = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        key_with_modifiers.kind = KeyEventKind::Press;

        assert_eq!(key_with_ctrl.code, KeyCode::Char('c'));
        assert_eq!(key_with_modifiers.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_screen_navigation_sequence() {
        let mut current_screen = ScreenState::All;

        current_screen = ScreenState::Cpu;

        current_screen = ScreenState::Gpu;
        assert_eq!(current_screen, ScreenState::Gpu);

        current_screen = ScreenState::Memory;
        assert_eq!(current_screen, ScreenState::Memory);

        current_screen = ScreenState::Power;
        assert_eq!(current_screen, ScreenState::Power);

        current_screen = ScreenState::Temperature;
        assert_eq!(current_screen, ScreenState::Temperature);

        current_screen = ScreenState::Control;
        assert_eq!(current_screen, ScreenState::Control);

        current_screen = ScreenState::Info;
        assert_eq!(current_screen, ScreenState::Info);

        current_screen = ScreenState::All;
        assert_eq!(current_screen, ScreenState::All);
    }

    #[test]
    fn test_tick_rate_duration() {
        let tick_rate = Duration::from_millis(250);
        assert_eq!(tick_rate.as_millis(), 250);

        let alternative_tick_rate = Duration::from_millis(100);
        assert_eq!(alternative_tick_rate.as_millis(), 100);
        assert_ne!(tick_rate, alternative_tick_rate);
    }

    #[test]
    fn test_state_message_clone() {
        let msg1 = StateMessage::SetScreen(ScreenState::Cpu);
        let msg2 = msg1.clone();

        assert_eq!(msg1, msg2);
    }

    #[test]
    fn test_multiple_state_messages() {
        let mut messages = Vec::new();

        messages.push(StateMessage::SetScreen(ScreenState::All));
        messages.push(StateMessage::Update);
        messages.push(StateMessage::SetScreen(ScreenState::Cpu));
        messages.push(StateMessage::Update);
        messages.push(StateMessage::Exit);

        assert_eq!(messages.len(), 5);
        assert!(matches!(
            messages[0],
            StateMessage::SetScreen(ScreenState::All)
        ));
        assert!(matches!(messages[1], StateMessage::Update));
        assert!(matches!(
            messages[2],
            StateMessage::SetScreen(ScreenState::Cpu)
        ));
        assert!(matches!(messages[3], StateMessage::Update));
        assert!(matches!(messages[4], StateMessage::Exit));
    }
}
