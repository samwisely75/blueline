//! # Buffer Operations
//!
//! Handles text insertion, deletion, and buffer content manipulation.
//!
//! HIGH-LEVEL LOGIC FLOW:
//! This module provides the core text editing operations for the REPL editor.
//! All operations are mode-aware and pane-aware, ensuring text modification
//! only occurs in appropriate contexts (Insert mode, Request pane).
//!
//! ARCHITECTURAL PATTERN:
//! - Operations validate mode and pane context before execution
//! - All changes go through PaneManager for proper event emission
//! - ViewEvents are emitted for selective rendering optimization
//! - Character-by-character processing maintains semantic consistency

use crate::repl::events::EditorMode;
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get selected text from current pane
    pub fn get_selected_text(&self) -> Option<String> {
        self.pane_manager.get_selected_text()
    }

    /// Yank text to yank buffer
    pub fn yank_to_buffer(&mut self, text: String) -> Result<()> {
        self.yank_buffer.yank(text)
    }
    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in Request pane and insert mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Use semantic insertion from PaneManager (handles visibility and all events)
        let events = self.pane_manager.insert_char_in_request(ch);
        self.emit_view_event(events)?;

        Ok(())
    }

    /// Insert text at current cursor position
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Only allow text insertion in Request pane and insert mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Insert each character individually to maintain proper semantic handling
        for ch in text.chars() {
            self.insert_char(ch)?;
        }

        Ok(())
    }

    /// Delete character before cursor
    pub fn delete_char_before_cursor(&mut self) -> Result<()> {
        let current_mode = self.mode();
        let is_request_pane = self.is_in_request_pane();

        tracing::debug!(
            "🗑️  delete_char_before_cursor: mode={:?}, is_request_pane={}",
            current_mode,
            is_request_pane
        );

        // Only allow deletion in Request pane and insert mode
        if !is_request_pane || current_mode != EditorMode::Insert {
            tracing::debug!(
                "🚫 Delete operation blocked: mode={:?}, is_request_pane={}",
                current_mode,
                is_request_pane
            );
            return Ok(());
        }

        tracing::debug!("✅ Delete operation allowed, proceeding with deletion");

        // Use semantic deletion from PaneManager
        let events = self.pane_manager.delete_char_before_cursor_in_request();
        tracing::debug!(
            "🗑️  PaneManager returned {} events from delete operation",
            events.len()
        );

        self.emit_view_event(events)?;

        tracing::debug!("🗑️  delete_char_before_cursor completed successfully");
        Ok(())
    }

    /// Delete character after cursor or empty line
    pub fn delete_char_after_cursor(&mut self) -> Result<()> {
        // Only allow deletion in Request pane and insert mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Use semantic deletion from PaneManager
        let events = self.pane_manager.delete_char_after_cursor_in_request();
        self.emit_view_event(events)?;

        Ok(())
    }
}
