// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Info screen - hardware information

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleBoardInfo {
    pub model: String,
    pub jetpack: String,
    pub l4t: String,
}

use super::{
    SimpleCpuStats, SimpleFanStats, SimpleGpuStats, SimpleMemoryStats, SimplePowerStats,
    SimpleTemperatureStats,
};

/// Info screen - hardware information
pub struct InfoScreen {
    stats: Option<InfoStats>,
}

#[derive(Debug, Clone)]
pub struct InfoStats {
    pub board: SimpleBoardInfo,
    pub cpu_cores: usize,
    pub cpu_governor: String,
    pub gpu_name: String,
}

impl InfoScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: InfoStats) {
        self.stats = Some(stats);
    }

    pub fn draw(&mut self, f: &mut Frame) {
        if let Some(stats) = &self.stats {
            self.draw_content(f, stats);
        } else {
            self.draw_loading(f);
        }
    }

    fn draw_loading(&self, f: &mut Frame) {
        let size = f.size();
        let paragraph = Paragraph::new("Loading...")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Info"));
        f.render_widget(paragraph, size);
    }

    fn draw_content(&self, f: &mut Frame, stats: &InfoStats) {
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

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "rusted-jetsons",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Info", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body(&self, f: &mut Frame, stats: &InfoStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(15), // Board info
                Constraint::Length(10), // CPU info
                Constraint::Length(10), // GPU info
                Constraint::Min(0),     // Spacer
            ])
            .split(area);

        self.draw_board_info(f, stats, body_chunks[0]);
        self.draw_cpu_info(f, stats, body_chunks[1]);
        self.draw_gpu_info(f, stats, body_chunks[2]);
    }

    fn draw_board_info(&self, f: &mut Frame, stats: &InfoStats, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "Board Information",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Model: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.board.model.as_str()),
            ]),
            Line::from(vec![
                Span::styled("Jetpack: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.board.jetpack.as_str()),
            ]),
            Line::from(vec![
                Span::styled("L4T: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.board.l4t.as_str()),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Board"));
        f.render_widget(paragraph, area);
    }

    fn draw_cpu_info(&self, f: &mut Frame, stats: &InfoStats, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "CPU Information",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Cores: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.cpu_cores.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Governor: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.cpu_governor.as_str()),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("CPU"));
        f.render_widget(paragraph, area);
    }

    fn draw_gpu_info(&self, f: &mut Frame, stats: &InfoStats, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "GPU Information",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Device: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.gpu_name.as_str()),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("GPU"));
        f.render_widget(paragraph, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = "q: quit | 1-8: screens | h: help";
        let paragraph = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for InfoScreen {
    fn default() -> Self {
        Self::new()
    }
}
