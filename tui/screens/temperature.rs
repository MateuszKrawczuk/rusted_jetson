// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Temperature screen - detailed temperature monitoring

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Frame,
};

use crate::modules::TemperatureStats;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleTemperatureStats {
    pub cpu: f32,
    pub gpu: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemperatureScreenStats {
    pub temperature: SimpleTemperatureStats,
    pub zones: Vec<ThermalZone>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThermalZone {
    pub name: String,
    pub current_temp: f32,
    pub max_temp: f32,
    pub critical_temp: f32,
    pub usage_percent: u16,
}

/// Temperature screen - detailed temperature monitoring
pub struct TemperatureScreen {
    stats: Option<TemperatureScreenStats>,
}

impl TemperatureScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: TemperatureScreenStats) {
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
            .block(Block::default().borders(Borders::ALL).title("Temperature"));
        f.render_widget(paragraph, size);
    }

    fn draw_content<B: Backend>(
        &self,
        f: &mut Frame<B>,
        stats: &TemperatureScreenStats,
        area: Rect,
    ) {
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

    fn draw_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "rusted-jetsons",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Temperature Details", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body<B: Backend>(&self, f: &mut Frame<B>, stats: &TemperatureScreenStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // Main temps
                Constraint::Min(0),     // All zones
            ])
            .split(area);

        self.draw_main_temps(f, stats, body_chunks[0]);
        self.draw_all_zones(f, stats, body_chunks[1]);
    }

    fn draw_main_temps<B: Backend>(
        &self,
        f: &mut Frame<B>,
        stats: &TemperatureScreenStats,
        area: Rect,
    ) {
        let items = vec![
            ListItem::new(format!("CPU: {:.1}°C", stats.temperature.cpu)),
            ListItem::new(format!("GPU: {:.1}°C", stats.temperature.gpu)),
            ListItem::new(""),
            ListItem::new("Temperature graph not implemented yet"),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Main Temperatures"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_all_zones<B: Backend>(&self, f: &mut Frame<B>, stats: &TemperatureScreen, area: Rect) {
        let items: Vec<ListItem> = stats
            .zones
            .iter()
            .map(|zone| {
                ListItem::new(format!(
                    "{:18} {:.1}°C / {:.1}°C ({}%)",
                    zone.name, zone.current_temp, zone.max_temp, zone.usage_percent
                ))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("All Thermal Zones"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_footer<B: Backend>(
        &self,
        f: &mut Frame<B>,
        stats: &TemperatureScreenStats,
        area: Rect,
    ) {
        let footer_text = format!(
            "q: quit | 1-8: screens | h: help | CPU: {:.1}°C | GPU: {:.1}°C",
            stats.temperature.cpu, stats.temperature.gpu
        );
        let paragraph = Paragraph::new(footer_text.as_str())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for TemperatureScreen {
    fn default() -> Self {
        Self::new()
    }
}
