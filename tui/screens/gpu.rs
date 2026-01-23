// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! GPU screen - detailed GPU monitoring

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::modules::{GpuStats, TemperatureStats};

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct SimpleGpuStats {
    pub usage: f32,
    pub frequency: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuScreenStats {
    pub gpu: SimpleGpuStats,
    pub temperature: crate::tui::screens::SimpleTemperatureStats,
    pub gpu_name: String,
    pub gpu_arch: String,
    pub memory_used: u64,
    pub memory_total: u64,
    pub state: String,
    pub governor: String,
    pub active_functions: Vec<String>,
}

impl Default for GpuScreenStats {
    fn default() -> Self {
        Self {
            gpu: SimpleGpuStats::default(),
            temperature: crate::tui::screens::SimpleTemperatureStats { cpu: 0.0, gpu: 0.0, board: 0.0 },
            gpu_name: "NVIDIA GPU".to_string(),
            gpu_arch: "Unknown".to_string(),
            memory_used: 0,
            memory_total: 0,
            state: String::new(),
            governor: String::new(),
            active_functions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuScreen {
    stats: Option<GpuScreenStats>,
}

impl GpuScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: GpuScreenStats) {
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
            .block(Block::default().borders(Borders::ALL).title("GPU"));
        f.render_widget(paragraph, size);
    }

    fn draw_content(&self, f: &mut Frame, stats: &GpuScreenStats) {
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
        self.draw_footer(f, stats, chunks[2]);
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
            Span::styled("GPU Details", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // Info
                Constraint::Min(0),     // Graph
            ])
            .split(area);

        self.draw_gpu_info(f, stats, body_chunks[0]);
        self.draw_usage_graph(f, stats, body_chunks[1]);
    }

    fn draw_gpu_info(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Usage gauge
                Constraint::Length(7), // Details
                Constraint::Length(3), // Temperature
                Constraint::Min(0),    // Info
            ])
            .split(area);

        self.draw_usage_gauge(f, stats, info_chunks[0]);
        self.draw_details(f, stats, info_chunks[1]);
        self.draw_temperature(f, stats, info_chunks[2]);
        self.draw_gpu_name(f, stats, info_chunks[3]);
    }

    fn draw_usage_gauge(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("GPU Usage"))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(stats.gpu.usage as u16)
            .label(format!("{}%", stats.gpu.usage));
        f.render_widget(gauge, area);
    }

    fn draw_details(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let text = vec![
            Line::from(vec![Span::styled(
                "GPU Details",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.gpu_name.as_str()),
            ]),
            Line::from(vec![
                Span::styled("Arch: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.gpu_arch.as_str()),
            ]),
            Line::from(vec![
                Span::styled("Freq: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{} MHz", stats.gpu.frequency / 1_000_000)),
            ]),
            Line::from(vec![
                Span::styled("Governor: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.governor.as_str()),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::Cyan)),
                Span::raw(if stats.state.is_empty() {
                    "N/A".to_string()
                } else {
                    stats.state.clone()
                }),
            ]),
            Line::from(vec![
                Span::styled("Mem: ", Style::default().fg(Color::Cyan)),
                Span::raw(if stats.memory_total > 0 {
                    format!(
                        "{} / {} MB",
                        stats.memory_used / 1024 / 1024,
                        stats.memory_total / 1024 / 1024
                    )
                } else {
                    "N/A".to_string()
                }),
            ]),
            Line::from(vec![
                Span::styled("Functions: ", Style::default().fg(Color::Cyan)),
                Span::raw(if stats.active_functions.is_empty() {
                    "None".to_string()
                } else {
                    stats.active_functions.join(", ")
                }),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(paragraph, area);
    }

    fn draw_temperature(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "GPU Temperature",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("GPU: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:.1}°C", stats.temperature.gpu)),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Temperature"));
        f.render_widget(paragraph, area);
    }

    fn draw_gpu_name(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
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
            Line::from(vec![
                Span::styled("Governor: ", Style::default().fg(Color::Cyan)),
                Span::raw(stats.governor.as_str()),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("GPU Info"));
        f.render_widget(paragraph, area);
    }

    fn draw_usage_graph(&self, f: &mut Frame, _stats: &GpuScreenStats, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "GPU Usage History",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Usage history not implemented yet"),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("History"))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn draw_footer(&self, f: &mut Frame, stats: &GpuScreenStats, area: Rect) {
        let footer_text = format!(
            "q: quit | 1-8: screens | h: help | GPU: {:.1}°C",
            stats.temperature.gpu
        );
        let paragraph = Paragraph::new(footer_text.as_str())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for GpuScreen {
    fn default() -> Self {
        Self::new()
    }
}
