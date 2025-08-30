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
pub use events::{ModelEvent, YankType};
pub use registry::UnifiedCommandRegistry;

// Re-export command implementations for registry
pub use change_selection::ChangeSelectionCommand;
pub use cut_character::CutCharacterCommand;
pub use cut_current_line::CutCurrentLineCommand;
pub use cut_selection::CutSelectionCommand;
pub use cut_to_end_of_line::CutToEndOfLineCommand;
pub use delete_selection::DeleteSelectionCommand;
pub use exit_visual_block_insert::ExitVisualBlockInsertCommand;
pub use http::HttpExecuteCommand;
pub use repeat_visual_selection::RepeatVisualSelectionCommand;
pub use visual_block_append::VisualBlockAppendCommand;
pub use visual_block_insert::VisualBlockInsertCommand;
pub use yank::YankSelectionCommand;
pub use yank_current_line::YankCurrentLineCommand;
