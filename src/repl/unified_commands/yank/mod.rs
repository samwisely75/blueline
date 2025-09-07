//! # Yank/Paste Commands Module
//!
//! This module contains all yank (copy) and paste related commands.
//! These commands handle copying text to clipboard/yank buffer and
//! pasting text from clipboard/yank buffer.
//!
//! NOTE: Commands self-register via inventory system - adding new command files
//! here won't cause merge conflicts. Add modules alphabetically to prevent conflicts.

// Yank commands
pub mod yank_selection;

// Paste commands
pub mod paste;
pub mod paste_at_cursor;

// Re-export all command structs using wildcards to prevent merge conflicts
pub use paste::*;
pub use paste_at_cursor::*;
pub use yank_selection::*;
