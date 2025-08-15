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

    /// Delete selected text from current pane
    /// Returns the deleted text if successful
    pub fn delete_selected_text(&mut self) -> Result<Option<String>> {
        if let Some((deleted_text, events)) = self.pane_manager.delete_selected_text() {
            self.emit_view_event(events)?;
            Ok(Some(deleted_text))
        } else {
            Ok(None)
        }
    }

    /// Yank text to yank buffer
    pub fn yank_to_buffer(&mut self, text: String) -> Result<()> {
        self.yank_buffer.yank(text)
    }

    /// Get text from yank buffer
    pub fn get_yanked_text(&mut self) -> Option<String> {
        self.yank_buffer.paste().map(|s| s.to_string())
    }

    /// Paste text at current cursor position (for P command)
    pub fn paste_text(&mut self, text: &str) -> Result<()> {
        // Only allow pasting in Request pane
        if !self.is_in_request_pane() {
            return Ok(());
        }

        // Temporarily switch to Insert mode for the paste operation
        let original_mode = self.mode();
        self.change_mode(EditorMode::Insert)?;

        // Insert each character
        for ch in text.chars() {
            let events = self.pane_manager.insert_char_in_request(ch);
            self.emit_view_event(events)?;
        }

        // Switch back to original mode
        self.change_mode(original_mode)?;

        Ok(())
    }

    /// Paste text after current cursor position (for paste after - p)
    pub fn paste_text_after(&mut self, text: &str) -> Result<()> {
        // Only allow pasting in Request pane
        if !self.is_in_request_pane() {
            return Ok(());
        }

        // Get current cursor position
        let current_pos = self.get_cursor_position();

        // In Normal mode, cursor sits ON a character. We want to insert AFTER that character.
        // So we need to insert at position current_column + 1
        // This is different from moving the cursor - we're just inserting at a different position

        // Temporarily switch to Insert mode for the paste operation
        let original_mode = self.mode();
        self.change_mode(EditorMode::Insert)?;

        // Move cursor to the position after the current character
        // We need to be careful here - if we're at the end of the line,
        // we should append at the end
        let line_length = self.pane_manager.get_current_line_length();
        if current_pos.column < line_length {
            // We're not at the end, move one position right for insertion
            let _ = self.move_cursor_right();
        }
        // If we're at or beyond the line length, cursor is already at the right position for append

        // Insert each character
        for ch in text.chars() {
            let events = self.pane_manager.insert_char_in_request(ch);
            self.emit_view_event(events)?;
        }

        // Switch back to original mode
        self.change_mode(original_mode)?;

        Ok(())
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

    /// Convert all tab characters to spaces in the request buffer
    /// Called when expandtab is enabled
    pub fn convert_tabs_to_spaces(&mut self) -> Result<()> {
        // Get the current request text
        let request_text = self.get_request_text();

        // Get the current tab width
        let tab_width = self.pane_manager.get_tab_width();

        // Replace all tabs with spaces
        let spaces = " ".repeat(tab_width);
        let converted_text = request_text.replace('\t', &spaces);

        // Only update if there were actual changes
        if converted_text != request_text {
            // Update the request buffer with the converted text
            let events = self.pane_manager.set_request_content(&converted_text);
            self.emit_view_event(events)?;
        }

        Ok(())
    }
}
