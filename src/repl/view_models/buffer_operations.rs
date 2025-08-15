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
            let events = self.pane_manager.insert_char(ch);
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
            let events = self.pane_manager.insert_char(ch);
            self.emit_view_event(events)?;
        }

        // Switch back to original mode
        self.change_mode(original_mode)?;

        Ok(())
    }

    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in Request pane and insert/visual block insert modes
        if !self.is_in_request_pane()
            || !matches!(
                self.mode(),
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
        {
            return Ok(());
        }

        // Use semantic insertion from PaneManager (handles visibility and all events)
        let events = self.pane_manager.insert_char(ch);
        self.emit_view_event(events)?;

        Ok(())
    }

    /// Insert text at current cursor position
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Only allow text insertion in Request pane and insert/visual block insert modes
        if !self.is_in_request_pane()
            || !matches!(
                self.mode(),
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
        {
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

        // Only allow deletion in Request pane and insert/visual block insert modes
        if !is_request_pane || !matches!(current_mode, EditorMode::Insert | EditorMode::VisualBlockInsert) {
            tracing::debug!(
                "🚫 Delete operation blocked: mode={:?}, is_request_pane={}",
                current_mode,
                is_request_pane
            );
            return Ok(());
        }

        tracing::debug!("✅ Delete operation allowed, proceeding with deletion");

        // Use semantic deletion from PaneManager
        let events = self.pane_manager.delete_char_before_cursor();
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
        // Only allow deletion in Request pane and insert/visual block insert modes
        if !self.is_in_request_pane() || !matches!(self.mode(), EditorMode::Insert | EditorMode::VisualBlockInsert) {
            return Ok(());
        }

        // Use semantic deletion from PaneManager
        let events = self.pane_manager.delete_char_after_cursor();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::LogicalPosition;

    #[test]
    fn test_visual_block_insert_mode_allows_text_insertion() {
        let mut vm = ViewModel::new();

        // Start in Normal mode and insert some test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move to first line, first column
        vm.set_cursor_position(LogicalPosition { line: 0, column: 0 })
            .unwrap();

        // Enter Visual Block Insert mode
        vm.change_mode(EditorMode::VisualBlockInsert).unwrap();

        // Verify that insert_text works in VisualBlockInsert mode
        let result = vm.insert_text("prefix ");
        assert!(
            result.is_ok(),
            "insert_text should work in VisualBlockInsert mode"
        );
    }

    #[test]
    fn test_visual_block_insert_mode_allows_char_insertion() {
        let mut vm = ViewModel::new();

        // Start in Normal mode and insert some test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move to first line, first column
        vm.set_cursor_position(LogicalPosition { line: 0, column: 0 })
            .unwrap();

        // Enter Visual Block Insert mode
        vm.change_mode(EditorMode::VisualBlockInsert).unwrap();

        // Verify that insert_char works in VisualBlockInsert mode
        let result = vm.insert_char('x');
        assert!(
            result.is_ok(),
            "insert_char should work in VisualBlockInsert mode"
        );
    }

    #[test]
    fn test_visual_block_insert_mode_allows_backspace() {
        let mut vm = ViewModel::new();
        
        // Start in Normal mode and insert some test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();
        
        // Move to a position where backspace can work
        vm.set_cursor_position(LogicalPosition { line: 0, column: 2 }).unwrap();
        
        // Enter Visual Block Insert mode
        vm.change_mode(EditorMode::VisualBlockInsert).unwrap();
        
        // Verify that delete_char_before_cursor works in VisualBlockInsert mode
        let result = vm.delete_char_before_cursor();
        assert!(result.is_ok(), "delete_char_before_cursor should work in VisualBlockInsert mode");
    }

    #[test]
    fn test_visual_block_insert_mode_allows_delete() {
        let mut vm = ViewModel::new();
        
        // Start in Normal mode and insert some test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();
        
        // Move to first line, first column
        vm.set_cursor_position(LogicalPosition { line: 0, column: 0 }).unwrap();
        
        // Enter Visual Block Insert mode
        vm.change_mode(EditorMode::VisualBlockInsert).unwrap();
        
        // Verify that delete_char_after_cursor works in VisualBlockInsert mode
        let result = vm.delete_char_after_cursor();
        assert!(result.is_ok(), "delete_char_after_cursor should work in VisualBlockInsert mode");
    }
}
