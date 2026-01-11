// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! TUI screens module

pub mod all;
pub mod control;
pub mod info;

pub use all::AllScreen;
pub use control::ControlScreen;
pub use info::InfoScreen;
