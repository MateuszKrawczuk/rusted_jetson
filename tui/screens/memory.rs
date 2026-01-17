// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Memory screen - detailed memory monitoring

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::modules::MemoryStats;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimpleMemoryStats {
    pub ram_used: u64,
    pub ram_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

/// Memory screen - detailed memory monitoring
pub struct MemoryScreen {
    stats: Option<MemoryScreenStats>,
}

#[derive(Debug, Clone)]
pub struct MemoryScreenStats {
    pub memory: SimpleMemoryStats,
    pub full_memory: MemoryStats,
}

impl MemoryScreen {
    pub fn new() -> Self {
        Self { stats: None }
    }

    pub fn update(&mut self, stats: MemoryScreenStats) {
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
            .block(Block::default().borders(Borders::ALL).title("Memory"));
        f.render_widget(paragraph, size);
    }

    fn draw_content(&self, f: &mut Frame, stats: &MemoryScreenStats) {
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
            Span::styled("Memory Details", Style::default().fg(Color::Gray)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(header, area);
    }

    fn draw_body(&self, f: &mut Frame, stats: &MemoryScreenStats, area: Rect) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // Memory bars
                Constraint::Min(0),     // Details
            ])
            .split(area);

        self.draw_memory_bars(f, stats, body_chunks[0]);
        self.draw_memory_details(f, stats, body_chunks[1]);
    }

    fn draw_memory_bars(&self, f: &mut Frame, stats: &MemoryScreenStats, area: Rect) {
        let mem_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35), // RAM
                Constraint::Percentage(35), // SWAP
                Constraint::Percentage(25), // IRAM
                Constraint::Min(0),         // Spacer
            ])
            .split(area);

        // RAM gauge
        let ram_percent = if stats.memory.ram_total > 0 {
            (stats.memory.ram_used * 100 / stats.memory.ram_total) as u16
        } else {
            0
        };

        let (ram_used_val, ram_used_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.ram_used);
        let (ram_total_val, ram_total_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.ram_total);

        let ram_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("RAM"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(ram_percent)
            .label(format!(
                "{:.1}{} / {:.1}{}",
                ram_used_val, ram_used_unit, ram_total_val, ram_total_unit
            ));
        f.render_widget(ram_gauge, mem_chunks[0]);

        // SWAP gauge
        let swap_percent = if stats.memory.swap_total > 0 {
            (stats.memory.swap_used * 100 / stats.memory.swap_total) as u16
        } else {
            0
        };

        let (swap_used_val, swap_used_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.swap_used);
        let (swap_total_val, swap_total_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.swap_total);

        let swap_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("SWAP"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(swap_percent)
            .label(format!(
                "{:.1}{} / {:.1}{}",
                swap_used_val, swap_used_unit, swap_total_val, swap_total_unit
            ));
        f.render_widget(swap_gauge, mem_chunks[1]);

        // IRAM gauge
        let iram_total = stats.full_memory.iram_total;
        if iram_total > 0 {
            let iram_used = stats.full_memory.iram_used;
            let iram_percent = (iram_used * 100 / iram_total) as u16;

            let (iram_used_val, iram_used_unit) =
                crate::modules::memory::format_memory_bytes(iram_used);
            let (iram_total_val, iram_total_unit) =
                crate::modules::memory::format_memory_bytes(iram_total);

            let iram_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("IRAM"))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(iram_percent)
                .label(format!(
                    "{:.1}{} / {:.1}{}",
                    iram_used_val, iram_used_unit, iram_total_val, iram_total_unit
                ));
            f.render_widget(iram_gauge, mem_chunks[2]);
        }
    }

    fn draw_memory_details(&self, f: &mut Frame, stats: &MemoryScreenStats, area: Rect) {
        let (ram_used_val, ram_used_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.ram_used);
        let (ram_total_val, ram_total_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.ram_total);
        let (swap_used_val, swap_used_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.swap_used);
        let (swap_total_val, swap_total_unit) =
            crate::modules::memory::format_memory_bytes(stats.memory.swap_total);
        let (ram_cached_val, ram_cached_unit) =
            crate::modules::memory::format_memory_bytes(stats.full_memory.ram_cached);
        let (swap_cached_val, swap_cached_unit) =
            crate::modules::memory::format_memory_bytes(stats.full_memory.swap_cached);
        let (iram_used_val, iram_used_unit) =
            crate::modules::memory::format_memory_bytes(stats.full_memory.iram_used);
        let (iram_total_val, iram_total_unit) =
            crate::modules::memory::format_memory_bytes(stats.full_memory.iram_total);
        let (iram_lfb_val, iram_lfb_unit) =
            crate::modules::memory::format_memory_bytes(stats.full_memory.iram_lfb);

        let items = vec![
            ListItem::new(format!(
                "RAM: {:.1}{} / {:.1}{}",
                ram_used_val, ram_used_unit, ram_total_val, ram_total_unit
            )),
            ListItem::new(format!(
                "SWAP: {:.1}{} / {:.1}{}",
                swap_used_val, swap_used_unit, swap_total_val, swap_total_unit
            )),
            ListItem::new(format!(
                "RAM Cached: {:.1}{}",
                ram_cached_val, ram_cached_unit
            )),
            ListItem::new(format!(
                "SWAP Cached: {:.1}{}",
                swap_cached_val, swap_cached_unit
            )),
            ListItem::new(format!(
                "IRAM: {:.1}{} / {:.1}{}",
                iram_used_val, iram_used_unit, iram_total_val, iram_total_unit
            )),
            ListItem::new(format!("IRAM LFB: {:.1}{}", iram_lfb_val, iram_lfb_unit)),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Memory Details"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(list, area);
    }

    fn draw_footer(&self, f: &mut Frame, stats: &MemoryScreenStats, area: Rect) {
        let footer_text = "q: quit | 1-8: screens | h: help";
        let paragraph = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for MemoryScreen {
    fn default() -> Self {
        Self::new()
    }
}
