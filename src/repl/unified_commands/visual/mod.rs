//! # Visual Mode Commands Module
//!
//! This module contains all visual mode related commands for the unified command system.
//! These commands handle various visual mode operations including visual block insert/append
//! operations, visual selection manipulation, and mode transitions.

pub mod exit_visual_block_insert;
pub mod repeat_visual_selection;
pub mod visual_block_append;
pub mod visual_block_insert;

// Re-export all commands using wildcards to prevent merge conflicts
pub use exit_visual_block_insert::*;
pub use repeat_visual_selection::*;
pub use visual_block_append::*;
pub use visual_block_insert::*;
