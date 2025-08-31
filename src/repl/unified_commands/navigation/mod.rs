//! # Navigation Commands Module
//!
//! This module contains all navigation-related commands that handle cursor
//! movement within the editor. These commands support both Vim-style keys
//! (h, j, k, l) and standard arrow keys.
//!
//! All commands in this module self-register using the inventory system
//! for dynamic discovery by the command registry.

// Navigation command modules
pub mod ex_goto_line;
pub mod move_left;
pub mod move_right;
pub mod move_up;

// Re-export command types for convenience
pub use ex_goto_line::ExGotoLineCommand;
pub use move_left::MoveLeftCommand;
pub use move_right::MoveRightCommand;
pub use move_up::MoveUpCommand;
