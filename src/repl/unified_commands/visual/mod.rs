//! # Visual Mode Commands Module
//!
//! This module contains visual mode operation commands that manipulate selections
//! or perform operations on visual selections, but do NOT change modes.
//! Mode transition commands belong in the mode/ module.
//!
//! NOTE: Commands self-register via inventory system - adding new command files
//! here won't cause merge conflicts. Add modules alphabetically to prevent conflicts.

pub mod repeat_visual_selection;

// Re-export all commands using wildcards to prevent merge conflicts
pub use repeat_visual_selection::*;
