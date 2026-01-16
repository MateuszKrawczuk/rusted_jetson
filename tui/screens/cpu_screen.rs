// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! CPU screen - detailed CPU monitoring

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::modules::{CpuStats, FanStats, TemperatureStats};

use super::SimpleTemperatureStats;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleCpuStats {
    pub usage: f32,
    pub frequency: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleFanStats {
    pub speed: u8,
}

#[derive(Debug, Clone)]
pub struct CpuScreenStats {
    pub overall: SimpleCpuStats,
    pub cores: Vec<CoreStats>,
    pub fan: SimpleFanStats,
    pub temperature: SimpleTemperatureStats,
}

#[derive(Debug, Clone)]
pub struct CoreStats {
    pub index: usize,
    pub usage: f32,
    pub frequency: u32,
    pub governor: String,
}

/// CPU screen - detailed CPU monitoring
pub struct CpuScreen {
    stats: Option<CpuScreenStats>,
    selected_core: usize,
}

impl CpuScreen {
    pub fn new() -> Self {
        Self {
            stats: None,
            selected_core: 0,
        }
    }

    pub fn update(&mut self, stats: CpuScreenStats) {
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
            .block(Block::default().borders(Borders::ALL).title("CPU"));
        f.render_widget(paragraph, size);
    }

    fn draw_content(&self, f: &mut Frame, stats: &CpuScreenStats) {
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
            Span::styled("CPU Details", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body(&self, f: &mut Frame, stats: &CpuScreenStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Core list
                Constraint::Min(0),     // Core details
            ])
            .split(area);

        self.draw_core_list(f, stats, body_chunks[0]);
        self.draw_core_details(f, stats, body_chunks[1]);
    }

    fn draw_core_list(&self, f: &mut Frame, stats: &CpuScreenStats, area: Rect) {
        let overall_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Overall CPU"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(stats.overall.usage as u16)
            .label(format!("{}%", stats.overall.usage));
        f.render_widget(overall_gauge, area);
    }

    fn draw_core_details(&self, f: &mut Frame, stats: &CpuScreenStats, area: Rect) {
        let items: Vec<ListItem> = stats
            .cores
            .iter()
            .map(|core| {
                ListItem::new(format!(
                    "Core {}: {}% @ {}MHz ({})",
                    core.index,
                    core.usage as u32,
                    core.frequency / 1_000_000,
                    core.governor
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("CPU Cores"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let fan_temp = if let Some(stats) = &self.stats {
            format!(
                "Fan: {}% | CPU: {:.1}°C",
                stats.fan.speed, stats.temperature.cpu
            )
        } else {
            "Loading...".to_string()
        };

        let footer_text = format!("q: quit | 1-8: screens | h: help | {}", fan_temp);
        let paragraph = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for CpuScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_screen_initialization() {
        let screen = CpuScreen::new();
        assert!(screen.stats.is_none());
        assert_eq!(screen.selected_core, 0);
    }

    #[test]
    fn test_cpu_screen_update() {
        let mut screen = CpuScreen::new();
        let test_stats = CpuScreenStats {
            overall: SimpleCpuStats {
                usage: 50.0,
                frequency: 2000,
            },
            cores: vec![],
            fan: SimpleFanStats { speed: 50 },
            temperature: SimpleTemperatureStats {
                cpu: 45.0,
                gpu: 50.0,
            },
        };

        screen.update(test_stats);
        assert!(screen.stats.is_some());
    }

    #[test]
    fn test_default() {
        let screen = CpuScreen::default();
        assert!(screen.stats.is_none());
        assert_eq!(screen.selected_core, 0);
    }
}
