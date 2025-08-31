//! # Visual Mode Commands Module
//!
//! This module contains all visual mode related commands for the unified command system.
//! These commands handle various visual mode operations including visual block insert/append
//! operations, visual selection manipulation, and mode transitions.

pub mod exit_visual_block_insert;
pub mod repeat_visual_selection;
pub mod visual_block_append;
pub mod visual_block_insert;

// Re-export the commands for convenience
pub use exit_visual_block_insert::ExitVisualBlockInsertCommand;
pub use repeat_visual_selection::RepeatVisualSelectionCommand;
pub use visual_block_append::VisualBlockAppendCommand;
pub use visual_block_insert::VisualBlockInsertCommand;
