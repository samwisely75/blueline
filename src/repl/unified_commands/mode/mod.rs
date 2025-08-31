//! # Mode Change Commands
//!
//! Commands for mode changes and related operations like append and insert positioning
//!
//! This module automatically includes all mode-related unified commands.
//! The dynamic discovery system handles command registration, so adding new commands
//! here won't cause merge conflicts.
//!
//! NOTE: Commands self-register via inventory system - adding new command files
//! here won't cause merge conflicts. Add modules alphabetically to prevent conflicts.

// Auto-include all command modules - add new ones alphabetically
pub mod append_after_cursor;
pub mod append_at_end_of_line;
pub mod enter_command_mode;
pub mod enter_insert_mode;
pub mod exit_insert_mode;
pub mod exit_visual_mode;
pub mod insert_at_beginning_of_line;

// Re-export all commands using wildcards to prevent merge conflicts
pub use append_after_cursor::*;
pub use append_at_end_of_line::*;
pub use enter_command_mode::*;
pub use enter_insert_mode::*;
pub use exit_insert_mode::*;
pub use exit_visual_mode::*;
pub use insert_at_beginning_of_line::*;
