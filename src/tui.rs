// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI module

pub mod app;
pub mod state;
pub mod widgets;
pub mod screens;

pub use app::TuiApp;
pub use state::{ScreenState, StateMessage};