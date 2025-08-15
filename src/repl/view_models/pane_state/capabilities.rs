//! Capability checking and mode management for PaneState
//!
//! This module contains methods for:
//! - Managing pane capabilities (EDITABLE, NAVIGABLE, etc.)
//! - Editor mode management
//! - Virtual column management for Vim-style navigation

use crate::repl::events::{EditorMode, PaneCapabilities};

use super::PaneState;

impl PaneState {
    /// Get the capabilities of this pane
    pub fn get_capabilities(&self) -> PaneCapabilities {
        self.capabilities
    }

    /// Check if this pane has a specific capability
    pub fn has_capability(&self, capability: PaneCapabilities) -> bool {
        self.capabilities.contains(capability)
    }

    /// Get the current editor mode for this pane
    pub fn get_mode(&self) -> EditorMode {
        self.editor_mode
    }

    /// Set editor mode for this pane
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.editor_mode = mode;
    }

    // ========================================
    // Virtual Column Management
    // ========================================
    // 
    // Virtual column maintains the desired cursor position for vertical movement,
    // allowing the cursor to "remember" its intended column when moving through
    // lines of different lengths (Vim-style behavior).

    /// Update virtual column to current cursor column (called when horizontal movement occurs)
    pub fn update_virtual_column(&mut self) {
        self.virtual_column = self.display_cursor.col;
    }

    /// Get the current virtual column
    pub fn get_virtual_column(&self) -> usize {
        self.virtual_column
    }

    /// Set virtual column explicitly (used for restoring desired position)
    pub fn set_virtual_column(&mut self, column: usize) {
        self.virtual_column = column;
    }
}