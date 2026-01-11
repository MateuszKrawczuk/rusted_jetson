// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! All screen - main dashboard

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::{JetsonStats, SimpleCpuStats, SimpleGpuStats, SimpleMemoryStats, SimpleFanStats, SimpleTemperatureStats, SimplePowerStats, SimpleBoardInfo};

/// All screen - main dashboard with all stats
pub struct AllScreen {
    stats: Option<JetsonStats>,
}

impl AllScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: JetsonStats) {
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("rusted-jetsons"),
            );
        f.render_widget(paragraph, size);
    }

    fn draw_content<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Footer
            ])
            .split(f.size());

        self.draw_header(f, chunks[0]);
        self.draw_body(f, stats, chunks[1]);
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "rusted-jetsons",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled(
                    "v0.1.0",
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),  // CPU
                Constraint::Length(10),  // GPU
                Constraint::Length(10),  // Memory
                Constraint::Length(10),  // Temperature
                Constraint::Length(10),  // Power
            ])
            .split(area);

        self.draw_cpu(f, stats, body_chunks[0]);
        self.draw_gpu(f, stats, body_chunks[1]);
        self.draw_memory(f, stats, body_chunks[2]);
        self.draw_temperature(f, stats, body_chunks[3]);
        self.draw_power(f, stats, body_chunks[4]);
    }

    fn draw_cpu<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CPU Usage"),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent(stats.cpu.usage as u16)
            .label(format!("{}%", stats.cpu.usage));
        f.render_widget(gauge, area);
    }

    fn draw_gpu<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GPU Usage"),
            )
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(stats.gpu.usage as u16)
            .label(format!("{}%", stats.gpu.usage));
        f.render_widget(gauge, area);
    }

    fn draw_memory<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats, area: Rect) {
        let ram_percent = if stats.memory.ram_total > 0 {
            (stats.memory.ram_used * 100 / stats.memory.ram_total) as u16
        } else {
            0
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Memory: {} MB / {} MB",
                        stats.memory.ram_used / 1024,
                        stats.memory.ram_total / 1024
                    )),
            )
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(ram_percent)
            .label(format!("{}%", ram_percent));
        f.render_widget(gauge, area);
    }

    fn draw_temperature<B: Backend>(
        &self,
        f: &mut Frame<B>,
        stats: &JetsonStats,
        area: Rect,
    ) {
        let board_temp = 0.0; // TODO: Implement board temp reading
        let text = format!(
            "CPU: {:.1}°C | GPU: {:.1}°C | Board: {:.1}°C",
            stats.temperature.cpu, stats.temperature.gpu, board_temp
        );

        let paragraph = Paragraph::new(text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Temperature"),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn draw_power<B: Backend>(&self, f: &mut Frame<B>, stats: &JetsonStats, area: Rect) {
        let text = format!("Total: {:.2}W", stats.power.total);

        let paragraph = Paragraph::new(text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Power Consumption"),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let footer_text = "q: quit | 1-8: screens | h: help";
        let paragraph = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for AllScreen {
    fn default() -> Self {
        Self::new()
    }
}
