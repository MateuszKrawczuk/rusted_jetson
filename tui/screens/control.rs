// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Control screen - hardware settings

use ratatui::{
    backend::Backend,
    crossterm::event::KeyEvent,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    SimpleBoardInfo, SimpleCpuStats, SimpleFanStats, SimpleGpuStats, SimpleMemoryStats,
    SimplePowerStats, SimpleTemperatureStats,
};

/// Control screen - hardware settings
pub struct ControlScreen {
    stats: Option<ControlStats>,
    selected_item: usize,
}

#[derive(Debug, Clone)]
struct ControlStats {
    pub fan_speed: u8,
    pub fan_mode: String,
    pub jetson_clocks: bool,
    pub jetson_clocks_status: String,
    pub nvpmodel_id: u8,
    pub nvpmodel_name: String,
}

impl ControlScreen {
    pub fn new() -> Self {
        Self {
            stats: None,
            selected_item: 0,
        }
    }

    pub fn update(&mut self, stats: ControlStats) {
        self.stats = Some(stats);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        if let Some(stats) = &self.stats {
            self.draw_content(f, stats);
        } else {
            self.draw_loading(f);
        }
    }

    fn draw_loading<B: Backend>(&self, f: &mut Frame<B>) {
        let size = f.size();
        let paragraph = Paragraph::new("Loading...")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Control"));
        f.render_widget(paragraph, size);
    }

    fn draw_content<B: Backend>(&self, f: &mut Frame<B>, stats: &ControlStats) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        self.draw_header(f, chunks[0]);
        self.draw_body(f, stats, chunks[1]);
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "rusted-jetsons",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Control", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body<B: Backend>(&self, f: &mut Frame<B>, stats: &ControlStats, area: Rect) {
        let items = vec![
            ListItem::new(format!(
                "Fan Speed: {}% ({})",
                stats.fan_speed, stats.fan_mode
            )),
            ListItem::new(format!(
                "Jetson Clocks: {} ({})",
                if stats.jetson_clocks { "ON" } else { "OFF" },
                stats.jetson_clocks_status
            )),
            ListItem::new(format!(
                "NVP Model: {} ({})",
                stats.nvpmodel_id, stats.nvpmodel_name
            )),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Hardware Control"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let footer_text = "q: quit | ↑↓: navigate | Enter: select | 1-8: screens | h: help";
        let paragraph = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: event::KeyEvent) -> anyhow::Result<()> {
        use event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Up => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_item < 2 {
                    self.selected_item += 1;
                }
            }
            KeyCode::Enter => {
                self.handle_select()?;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_select(&mut self) -> anyhow::Result<()> {
        match self.selected_item {
            0 => {
                // Fan speed control
                println!("Fan speed control not implemented yet");
            }
            1 => {
                // Toggle jetson_clocks
                println!("Jetson clocks control not implemented yet");
            }
            2 => {
                // NVP model selection
                println!("NVP model selection not implemented yet");
            }
            _ => {}
        }
        Ok(())
    }
}

impl Default for ControlScreen {
    fn default() -> Self {
        Self::new()
    }
}
