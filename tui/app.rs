// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI application structure

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
        execute,
    },
    Terminal,
};

use crate::JetsonStats;
use crate::tui::screens::AllScreen;
use crate::tui::state::{ScreenState, StateMessage};

/// Main TUI application
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend>,
    tx: mpsc::Sender<StateMessage>,
    rx: mpsc::Receiver<StateMessage>,
    current_screen: ScreenState,
    all_screen: AllScreen,
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
        // Generate mock stats for now
        let stats = self.generate_mock_stats();
        self.stats = Some(stats.clone());
        self.all_screen.update(stats);
        let _ = self.draw();
    }

    fn generate_mock_stats(&self) -> JetsonStats {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        JetsonStats {
            cpu: crate::CpuStats {
                usage: rng.gen_range(20.0..80.0),
                frequency: rng.gen_range(1000..2000),
            },
            gpu: crate::GpuStats {
                usage: rng.gen_range(10.0..70.0),
                frequency: rng.gen_range(500..1500),
            },
            memory: crate::MemoryStats {
                ram_used: rng.gen_range(2048..6144),
                ram_total: 8192,
                swap_used: rng.gen_range(0..1024),
                swap_total: 2048,
            },
            fan: crate::FanStats {
                speed: rng.gen_range(30..80),
            },
            temperature: crate::TemperatureStats {
                cpu: rng.gen_range(30.0..60.0),
                gpu: rng.gen_range(30.0..60.0),
            },
            power: crate::PowerStats {
                total: rng.gen_range(5.0..25.0),
            },
            board: crate::BoardInfo {
                model: "Jetson Xavier NX".to_string(),
                jetpack: "5.1.2".to_string(),
                l4t: "35.3.1".to_string(),
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
        self.terminal.draw(|f| {
            match self.current_screen {
                ScreenState::All => {
                    self.all_screen.draw(f);
                }
                _ => {
                    // TODO: Implement other screens
                    let text = format!("Screen: {:?} (not implemented yet)", self.current_screen);
                    let paragraph = ratatui::widgets::Paragraph::new(text.as_str())
                        .alignment(ratatui::layout::Alignment::Center)
                        .block(ratatui::widgets::Block::default().borders(
                            ratatui::widgets::Borders::ALL,
                        ));
                    f.render_widget(paragraph, f.size());
                }
            }
        })?;

        self.terminal.flush()?;
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Restore terminal
        let _ = execute!(
            io::stdout(),
            DisableMouseCapture,
        );
        let _ = self.terminal.show_cursor();
    }
}
