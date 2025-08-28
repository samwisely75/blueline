//! # ViewModel Module
//!
//! Modular ViewModel implementation split into focused responsibilities.
//! This replaces the monolithic view_models.rs with a clean, maintainable architecture.

mod app_view_model;
pub mod core;
// Most business logic modules moved to models/app_state/

// Re-export the new AppViewModel
pub use app_view_model::AppViewModel;

// Keep ViewModel as an alias for backward compatibility during migration
pub use core::ViewModel;

// Re-export types that other modules need
pub use app_view_model::DisplayLineData;
// PaneManager now imported from models/app_state
pub use crate::repl::models::app_state::PaneManager;
// PaneState now imported from models
pub use crate::repl::models::pane_state::PaneState;
// Selection and YankBuffer types now imported from models
pub use crate::repl::models::{Selection, YankEntry, YankType};
