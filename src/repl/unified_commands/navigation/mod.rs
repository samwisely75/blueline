//! # Navigation Commands Module
//!
//! This module contains all navigation-related commands that handle cursor
//! movement within the editor. These commands support both Vim-style keys
//! (h, j, k, l) and standard arrow keys.
//!
//! NOTE: Commands self-register via inventory system - adding new command files
//! here won't cause merge conflicts. Add modules alphabetically to prevent conflicts.

// Navigation command modules
pub mod cancel_g_prefix;
pub mod end_of_word;
pub mod ex_goto_line;
pub mod go_to_bottom;
pub mod go_to_top;
pub mod half_page_down;
pub mod move_down;
pub mod move_left;
pub mod move_right;
pub mod move_up;
pub mod next_word;
pub mod page_down;
pub mod previous_word;
pub mod scroll_left;
pub mod scroll_right;
pub mod switch_pane;

// Re-export all command types using wildcards to prevent merge conflicts
pub use cancel_g_prefix::*;
pub use end_of_word::*;
pub use ex_goto_line::*;
pub use go_to_bottom::*;
pub use go_to_top::*;
pub use half_page_down::*;
pub use move_down::*;
pub use move_left::*;
pub use move_right::*;
pub use move_up::*;
pub use next_word::*;
pub use page_down::*;
pub use previous_word::*;
pub use scroll_left::*;
pub use scroll_right::*;
pub use switch_pane::*;
