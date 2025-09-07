//! # Application State Model
//!
//! Contains the complete application state and its associated business logic.
//! Following the same pattern as PaneState, this module encapsulates both
//! data and operations on that data.

// Core data structure
mod core;
pub use core::{AppState, DisplayLineData};

// Domain context modules
mod editor_context;
mod http_context;
mod ui_context;

// Business logic modules
mod buffer_operations;
mod cursor_manager;
mod display_manager;
mod ex_command_manager;
mod http_manager;
mod mode_manager;
mod pane_manager;
mod rendering_coordinator;
mod settings_manager;

// Re-export PaneManager for backward compatibility
pub use pane_manager::PaneManager;
