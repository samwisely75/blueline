//! # Yank/Paste Commands Module
//!
//! This module contains all yank (copy) and paste related commands.
//! These commands handle copying text to clipboard/yank buffer and
//! pasting text from clipboard/yank buffer.

// Yank commands
pub mod yank_current_line;
pub mod yank_selection;

// Paste commands
pub mod paste;
pub mod paste_at_cursor;

// Re-export command structs for easier access
pub use paste::PasteAfterCommand;
pub use paste_at_cursor::PasteAtCursorCommand;
pub use yank_current_line::YankCurrentLineCommand;
pub use yank_selection::YankSelectionCommand;
