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
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_state_count() {
        assert_eq!(ScreenState::COUNT, 8, "Should have 8 screen states");
    }

    #[test]
    fn test_screen_state_from_index() {
        assert_eq!(ScreenState::from_index(0), Some(ScreenState::All));
        assert_eq!(ScreenState::from_index(1), Some(ScreenState::Cpu));
        assert_eq!(ScreenState::from_index(2), Some(ScreenState::Gpu));
        assert_eq!(ScreenState::from_index(3), Some(ScreenState::Memory));
        assert_eq!(ScreenState::from_index(4), Some(ScreenState::Power));
        assert_eq!(ScreenState::from_index(5), Some(ScreenState::Temperature));
        assert_eq!(ScreenState::from_index(6), Some(ScreenState::Control));
        assert_eq!(ScreenState::from_index(7), Some(ScreenState::Info));
        assert_eq!(
            ScreenState::from_index(8),
            None,
            "Index 8 should be out of range"
        );
        assert_eq!(
            ScreenState::from_index(999),
            None,
            "Index 999 should be out of range"
        );
    }

    #[test]
    fn test_screen_state_index() {
        assert_eq!(ScreenState::All.index(), 1);
        assert_eq!(ScreenState::Cpu.index(), 2);
        assert_eq!(ScreenState::Gpu.index(), 3);
        assert_eq!(ScreenState::Memory.index(), 4);
        assert_eq!(ScreenState::Power.index(), 5);
        assert_eq!(ScreenState::Temperature.index(), 6);
        assert_eq!(ScreenState::Control.index(), 7);
        assert_eq!(ScreenState::Info.index(), 8);
    }

    #[test]
    fn test_screen_state_name() {
        assert_eq!(ScreenState::All.name(), "All");
        assert_eq!(ScreenState::Cpu.name(), "CPU");
        assert_eq!(ScreenState::Gpu.name(), "GPU");
        assert_eq!(ScreenState::Memory.name(), "Memory");
        assert_eq!(ScreenState::Power.name(), "Power");
        assert_eq!(ScreenState::Temperature.name(), "Temperature");
        assert_eq!(ScreenState::Control.name(), "Control");
        assert_eq!(ScreenState::Info.name(), "Info");
    }

    #[test]
    fn test_screen_state_equality() {
        assert_eq!(ScreenState::All, ScreenState::All);
        assert_eq!(ScreenState::Cpu, ScreenState::Cpu);
        assert_ne!(ScreenState::Cpu, ScreenState::Gpu);
        assert_ne!(ScreenState::Memory, ScreenState::Power);
    }

    #[test]
    fn test_screen_state_copy_and_clone() {
        let state1 = ScreenState::Cpu;
        let state2 = state1;
        assert_eq!(state1, ScreenState::Cpu);
        assert_eq!(state2, ScreenState::Cpu);
    }

    #[test]
    fn test_state_message_set_screen() {
        let msg = StateMessage::SetScreen(ScreenState::Cpu);
        match msg {
            StateMessage::SetScreen(screen) => assert_eq!(screen, ScreenState::Cpu),
            _ => panic!("Expected SetScreen variant"),
        }
    }

    #[test]
    fn test_state_message_update() {
        let msg = StateMessage::Update;
        match msg {
            StateMessage::Update => (),
            _ => panic!("Expected Update variant"),
        }
    }

    #[test]
    fn test_state_message_exit() {
        let msg = StateMessage::Exit;
        match msg {
            StateMessage::Exit => (),
            _ => panic!("Expected Exit variant"),
        }
    }

    #[test]
    fn test_state_message_error() {
        let error_msg = "Test error".to_string();
        let msg = StateMessage::Error(error_msg.clone());
        match msg {
            StateMessage::Error(err) => assert_eq!(err, error_msg),
            _ => panic!("Expected Error variant"),
        }
    }
}
