//! # Editing Commands Module
//!
//! This module contains all editing-related commands that modify text content,
//! including cutting, deleting, and changing text selections.

// Editing command implementations
pub mod change_selection;
pub mod cut_character;
pub mod cut_current_line;
pub mod cut_selection;
pub mod cut_to_end_of_line;
pub mod delete_selection;
pub mod multi_cursor_text_insert;

// Re-export the command structs for easier access
pub use change_selection::ChangeSelectionCommand;
pub use cut_character::CutCharacterCommand;
pub use cut_current_line::CutCurrentLineCommand;
pub use cut_selection::CutSelectionCommand;
pub use cut_to_end_of_line::CutToEndOfLineCommand;
pub use delete_selection::DeleteSelectionCommand;
pub use multi_cursor_text_insert::MultiCursorTextInsertCommand;
