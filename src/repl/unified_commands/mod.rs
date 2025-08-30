//! # Unified Command Pattern Infrastructure  
//!
//! This module implements the new unified Command Pattern architecture that
//! replaces the old command system. Commands contain both key binding logic
//! (is_relevant) and business logic (handle), emitting semantic ModelEvents.

// Core infrastructure
pub mod command;
pub mod dynamic_registry; // Dynamic command discovery system
pub mod events;

// Command implementations
pub mod change_selection;
pub mod cut_character;
pub mod cut_current_line;
pub mod cut_selection;
pub mod cut_to_end_of_line;
pub mod delete_selection;
pub mod exit_visual_block_insert;
pub mod http;
pub mod repeat_visual_selection;
pub mod setting_change;
pub mod show_profile;
pub mod visual_block_append;
pub mod visual_block_insert;
pub mod yank;
pub mod yank_current_line;

// Re-export main types
pub use command::{Command, CommandContext, ExecutionContext};
pub use dynamic_registry::{register_all_commands, CommandEntry, CommandFactory, DynamicCommandRegistry};
pub use events::{ModelEvent, YankType};

// Note: Individual command re-exports no longer needed -
// commands self-register using the inventory system
