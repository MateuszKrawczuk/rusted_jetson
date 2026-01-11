// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI screen states

/// Screen state for TUI application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    All,
    Cpu,
    Gpu,
    Memory,
    Power,
    Temperature,
    Control,
    Info,
}

impl ScreenState {
    pub const COUNT: usize = 8;

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(ScreenState::All),
            1 => Some(ScreenState::Cpu),
            2 => Some(ScreenState::Gpu),
            3 => Some(ScreenState::Memory),
            4 => Some(ScreenState::Power),
            5 => Some(ScreenState::Temperature),
            6 => Some(ScreenState::Control),
            7 => Some(ScreenState::Info),
            _ => None,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            ScreenState::All => 1,
            ScreenState::Cpu => 2,
            ScreenState::Gpu => 3,
            ScreenState::Memory => 4,
            ScreenState::Power => 5,
            ScreenState::Temperature => 6,
            ScreenState::Control => 7,
            ScreenState::Info => 8,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ScreenState::All => "All",
            ScreenState::Cpu => "CPU",
            ScreenState::Gpu => "GPU",
            ScreenState::Memory => "Memory",
            ScreenState::Power => "Power",
            ScreenState::Temperature => "Temperature",
            ScreenState::Control => "Control",
            ScreenState::Info => "Info",
        }
    }
}

/// Message for communication between data collector and UI
#[derive(Debug, Clone)]
pub enum StateMessage {
    /// Update screen state
    SetScreen(ScreenState),
    /// Update with new stats
    Update,
    /// Exit application
    Exit,
    /// Error occurred
    Error(String),
}
