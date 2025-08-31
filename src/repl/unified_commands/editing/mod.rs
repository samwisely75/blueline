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
pub mod insert_char;
pub mod multi_cursor_text_insert;

// Re-export all command structs using wildcards to prevent merge conflicts
pub use change_selection::*;
pub use cut_character::*;
pub use cut_current_line::*;
pub use cut_selection::*;
pub use cut_to_end_of_line::*;
pub use delete_selection::*;
pub use insert_char::*;
pub use multi_cursor_text_insert::*;
