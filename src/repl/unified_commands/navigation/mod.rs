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
pub mod go_to_bottom;
pub mod go_to_top;
pub mod move_down;
pub mod move_left;
pub mod move_right;
pub mod move_up;
pub mod scroll_left;
pub mod scroll_right;

// Re-export all command types using wildcards to prevent merge conflicts
pub use ex_goto_line::*;
pub use go_to_bottom::*;
pub use go_to_top::*;
pub use move_down::*;
pub use move_left::*;
pub use move_right::*;
pub use move_up::*;
pub use scroll_left::*;
pub use scroll_right::*;
