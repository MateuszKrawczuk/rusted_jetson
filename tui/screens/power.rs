// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Power screen - detailed power monitoring

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
    Frame,
};

use crate::modules::PowerStats as FullPowerStats;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimplePowerStats {
    pub total: f32,
}

/// Power screen - detailed power monitoring
pub struct PowerScreen {
    stats: Option<PowerScreenStats>,
}

#[derive(Debug, Clone)]
pub struct PowerScreenStats {
    pub power: SimplePowerStats,
    pub rails: Vec<PowerRail>,
}

#[derive(Debug, Clone)]
pub struct PowerRail {
    pub name: String,
    pub current: f32,
    pub voltage: f32,
    pub power: f32,
}

impl PowerScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: PowerScreenStats) {
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
            .block(Block::default().borders(Borders::ALL).title("Power"));
        f.render_widget(paragraph, size);
    }

    fn draw_content(&self, f: &mut Frame, stats: &PowerScreenStats) {
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
            Span::styled("Power Details", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body(&self, f: &mut Frame, stats: &PowerScreenStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Total power
                Constraint::Min(0),     // Power rails list
            ])
            .split(area);

        self.draw_total_power(f, stats, body_chunks[0]);
        self.draw_power_rails(f, stats, body_chunks[1]);
    }

    fn draw_total_power(&self, f: &mut Frame, stats: &PowerScreenStats, area: Rect) {
        let items = vec![
            ListItem::new(format!("Total: {:.2}W", stats.power.total)),
            ListItem::new(""),
            ListItem::new("Power usage graph not implemented yet"),
        ];

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Total Power"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_power_rails(&self, f: &mut Frame, stats: &PowerScreenStats, area: Rect) {
        let items: Vec<ListItem> = stats
            .rails
            .iter()
            .map(|rail| {
                ListItem::new(format!(
                    "{:12} {:.2}mA {:.2}mV {:.2}mW",
                    rail.name, rail.current, rail.voltage, rail.power
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Power Rails"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_footer(&self, f: &mut Frame, stats: &PowerScreenStats, area: Rect) {
        let footer_text = format!(
            "q: quit | 1-8: screens | h: help | Total: {:.2}W",
            stats.power.total
        );
        let paragraph = Paragraph::new(footer_text.as_str())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for PowerScreen {
    fn default() -> Self {
        Self::new()
    }
}
