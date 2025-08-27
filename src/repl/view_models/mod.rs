//! # ViewModel Module
//!
//! Modular ViewModel implementation split into focused responsibilities.
//! This replaces the monolithic view_models.rs with a clean, maintainable architecture.

mod app_view_model;
mod buffer_operations;
mod core;
mod cursor_manager;
mod display_manager;
mod ex_command_manager;
mod http_manager;
mod mode_manager;
mod pane_manager;
// pane_state moved to models/state/
mod rendering_coordinator;
// screen_buffer moved to models/
// selection moved to models/
mod settings_manager;
// yank_buffer moved to models/

// Re-export the new AppViewModel
pub use app_view_model::AppViewModel;

// Keep ViewModel as an alias for backward compatibility during migration
pub use core::ViewModel;

// Re-export types that other modules need
pub use app_view_model::DisplayLineData;
pub use pane_manager::PaneManager;
// PaneState now imported from models
pub use crate::repl::models::state::pane_state::PaneState;
// Selection and YankBuffer types now imported from models
pub use crate::repl::models::{Selection, YankEntry, YankType};
