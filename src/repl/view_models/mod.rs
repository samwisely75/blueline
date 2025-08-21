//! # ViewModel Module
//!
//! Modular ViewModel implementation split into focused responsibilities.
//! This replaces the monolithic view_models.rs with a clean, maintainable architecture.

mod buffer_operations;
pub mod commands;
mod core;
mod cursor_manager;
mod display_manager;
mod ex_command_manager;
mod http_manager;
mod mode_manager;
mod pane_manager;
mod pane_state;
mod rendering_coordinator;
// screen_buffer moved to models/
mod selection;
mod settings_manager;
// yank_buffer moved to models/

// Re-export the main ViewModel
pub use core::ViewModel;

// Re-export types that other modules need
pub use core::DisplayLineData;
pub use pane_manager::PaneManager;
pub use pane_state::PaneState;
pub use selection::Selection;
// YankBuffer types now imported from models
pub use crate::repl::models::{YankEntry, YankType};
