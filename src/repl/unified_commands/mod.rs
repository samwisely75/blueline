//! # Unified Command Pattern Infrastructure  
//!
//! This module implements the new unified Command Pattern architecture that
//! replaces the old command system. Commands contain both key binding logic
//! (is_relevant) and business logic (handle), emitting semantic ModelEvents.

// Core infrastructure
pub mod command;
pub mod events;
pub mod registry;

// Command implementations
pub mod change_selection;
pub mod cut_character;
pub mod cut_current_line;
pub mod cut_selection;
pub mod cut_to_end_of_line;
pub mod delete_selection;
pub mod http;
pub mod setting_change;
pub mod show_profile;
pub mod visual_block_insert;
pub mod yank;
pub mod yank_current_line;

// Re-export main types
pub use command::{Command, CommandContext, ExecutionContext};
pub use events::{ModelEvent, YankType};
pub use registry::UnifiedCommandRegistry;
