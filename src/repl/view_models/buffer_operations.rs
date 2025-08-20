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

use crate::repl::events::{EditorMode, LogicalPosition, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use crate::repl::view_models::{YankEntry, YankType};
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

    /// Yank text to yank buffer with type information
    pub fn yank_to_buffer_with_type(&mut self, text: String, yank_type: YankType) -> Result<()> {
        self.yank_buffer.yank_with_type(text, yank_type)
    }

    /// Yank text to yank buffer (defaults to Character type for backward compatibility)
    pub fn yank_to_buffer(&mut self, text: String) -> Result<()> {
        self.yank_buffer.yank(text)
    }

    /// Get yank entry with type information from yank buffer
    pub fn get_yanked_entry(&mut self) -> Option<YankEntry> {
        self.yank_buffer.paste_entry()
    }

    /// Get text from yank buffer (for backward compatibility)
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

    /// Advanced paste operation that respects yank type (character, line, or block)
    pub fn paste_with_type(&mut self, yank_entry: &YankEntry) -> Result<()> {
        match yank_entry.yank_type {
            YankType::Character => self.paste_text(&yank_entry.text),
            YankType::Line => self.paste_line_wise(&yank_entry.text),
            YankType::Block => self.paste_block_wise(&yank_entry.text),
        }
    }

    /// Advanced paste after operation that respects yank type (character, line, or block)
    pub fn paste_after_with_type(&mut self, yank_entry: &YankEntry) -> Result<()> {
        match yank_entry.yank_type {
            YankType::Character => self.paste_text_after(&yank_entry.text),
            YankType::Line => self.paste_line_wise_after(&yank_entry.text),
            YankType::Block => self.paste_block_wise_after(&yank_entry.text),
        }
    }

    /// Paste text as lines (for line-wise yanks)
    pub fn paste_line_wise(&mut self, text: &str) -> Result<()> {
        // Only allow pasting in Request pane
        if !self.is_in_request_pane() {
            return Ok(());
        }

        // For line-wise paste at cursor (P), insert at beginning of current line
        let current_pos = self.get_cursor_position();
        let line_start = LogicalPosition {
            line: current_pos.line,
            column: 0,
        };

        // Move cursor to line start
        self.set_cursor_position(line_start)?;

        // Temporarily switch to Insert mode for the paste operation
        let original_mode = self.mode();
        self.change_mode(EditorMode::Insert)?;

        // Insert the text followed by a newline
        for ch in text.chars() {
            let events = self.pane_manager.insert_char(ch);
            self.emit_view_event(events)?;
        }

        // Add newline at the end
        let events = self.pane_manager.insert_char('\n');
        self.emit_view_event(events)?;

        // Switch back to original mode
        self.change_mode(original_mode)?;

        Ok(())
    }

    /// Paste text as lines after current line (for line-wise yanks with p command)
    pub fn paste_line_wise_after(&mut self, text: &str) -> Result<()> {
        // Only allow pasting in Request pane
        if !self.is_in_request_pane() {
            return Ok(());
        }

        // For line-wise paste after (p), move to end of current line and add newline
        let current_pos = self.get_cursor_position();
        let line_length = self.pane_manager.get_current_line_length();
        let line_end = LogicalPosition {
            line: current_pos.line,
            column: line_length,
        };

        // Move cursor to line end
        self.set_cursor_position(line_end)?;

        // Temporarily switch to Insert mode for the paste operation
        let original_mode = self.mode();
        self.change_mode(EditorMode::Insert)?;

        // Insert newline first, then the text
        let events = self.pane_manager.insert_char('\n');
        self.emit_view_event(events)?;

        for ch in text.chars() {
            let events = self.pane_manager.insert_char(ch);
            self.emit_view_event(events)?;
        }

        // Switch back to original mode
        self.change_mode(original_mode)?;

        Ok(())
    }

    /// Paste text in block-wise manner (rectangular paste maintaining column alignment)
    pub fn paste_block_wise(&mut self, text: &str) -> Result<()> {
        // Only allow pasting in Request pane
        if !self.is_in_request_pane() {
            return Ok(());
        }

        let current_pos = self.get_cursor_position();
        tracing::debug!(
            "paste_block_wise called at position {:?} with text: '{}'",
            current_pos,
            text
        );

        // Split the block text into lines
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok(());
        }

        tracing::debug!("Calling insert_block_wise with {} lines", lines.len());

        // Use the new block-wise insertion method that handles positioning correctly
        let events = self.pane_manager.insert_block_wise(current_pos, &lines);
        self.emit_view_event(events)?;

        Ok(())
    }

    /// Paste text in block-wise manner after cursor position
    pub fn paste_block_wise_after(&mut self, text: &str) -> Result<()> {
        // For block-wise paste after, move cursor one column right and paste
        let current_pos = self.get_cursor_position();
        let after_pos = LogicalPosition {
            line: current_pos.line,
            column: current_pos.column + 1,
        };

        self.set_cursor_position(after_pos)?;
        self.paste_block_wise(text)
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
        if !is_request_pane
            || !matches!(
                current_mode,
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
        {
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
        if !self.is_in_request_pane()
            || !matches!(
                self.mode(),
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
        {
            return Ok(());
        }

        // In Visual Block Insert mode, use restricted deletion (no line joining)
        let events = if self.mode() == EditorMode::VisualBlockInsert {
            self.pane_manager
                .delete_char_after_cursor_visual_block_safe()
        } else {
            self.pane_manager.delete_char_after_cursor()
        };
        self.emit_view_event(events)?;

        Ok(())
    }

    /// Cut (delete and yank) character at cursor position
    pub fn cut_char_at_cursor(&mut self) -> Result<()> {
        // Only allow in Request pane and Normal mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Normal {
            return Ok(());
        }

        // Delete the character and get it back for yanking
        if let Some(deleted_char) = self.pane_manager.cut_char_at_cursor() {
            // Yank the deleted character to the buffer
            self.yank_to_buffer_with_type(deleted_char, YankType::Character)?;

            // Emit view events for display update
            self.emit_view_event(vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ])?;
        }

        Ok(())
    }

    /// Cut from cursor to end of line and yank to buffer (D command)
    pub fn cut_to_end_of_line(&mut self) -> Result<()> {
        // Only allow in Request pane and Normal mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Normal {
            return Ok(());
        }

        // Delete from cursor to end of line and get the text for yanking
        if let Some(cut_text) = self.pane_manager.cut_to_end_of_line() {
            // Yank the cut text to the buffer as character type
            self.yank_to_buffer_with_type(cut_text, YankType::Character)?;

            // Emit view events for display update
            self.emit_view_event(vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ])?;
        }

        Ok(())
    }

    /// Cut entire current line and yank to buffer (dd command)
    pub fn cut_current_line(&mut self) -> Result<()> {
        // Only allow in Request pane and Normal mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Normal {
            return Ok(());
        }

        // Delete entire current line and get the text for yanking
        if let Some(cut_text) = self.pane_manager.cut_current_line() {
            // Yank the cut text to the buffer as line type (includes newline)
            self.yank_to_buffer_with_type(cut_text, YankType::Line)?;

            // Emit view events for display update
            self.emit_view_event(vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ])?;
        }

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
        vm.set_cursor_position(LogicalPosition { line: 0, column: 2 })
            .unwrap();

        // Enter Visual Block Insert mode
        vm.change_mode(EditorMode::VisualBlockInsert).unwrap();

        // Verify that delete_char_before_cursor works in VisualBlockInsert mode
        let result = vm.delete_char_before_cursor();
        assert!(
            result.is_ok(),
            "delete_char_before_cursor should work in VisualBlockInsert mode"
        );
    }

    #[test]
    fn test_visual_block_insert_mode_allows_delete() {
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

        // Verify that delete_char_after_cursor works in VisualBlockInsert mode
        let result = vm.delete_char_after_cursor();
        assert!(
            result.is_ok(),
            "delete_char_after_cursor should work in VisualBlockInsert mode"
        );
    }

    #[test]
    fn test_visual_selection_cleared_after_visual_block_insert() {
        let mut vm = ViewModel::new();

        // Start in Normal mode and insert some test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Enter Visual Block mode and start a selection
        vm.change_mode(EditorMode::VisualBlock).unwrap();
        let selection = vm.get_visual_selection();
        assert!(
            selection.0.is_some(),
            "Should have visual selection in VisualBlock mode"
        );

        // Clear visual selection (simulating exit from Visual Block Insert)
        let result = vm.clear_visual_selection();
        assert!(result.is_ok(), "clear_visual_selection should work");

        // Verify selection is cleared
        let selection_after = vm.get_visual_selection();
        assert!(
            selection_after.0.is_none(),
            "Visual selection should be cleared"
        );
    }

    #[test]
    fn test_cut_to_end_of_line_in_normal_mode() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move cursor to middle of line (position 6, after "hello ")
        vm.set_cursor_position(LogicalPosition { line: 0, column: 6 })
            .unwrap();

        // Cut from cursor to end of line
        let result = vm.cut_to_end_of_line();
        assert!(
            result.is_ok(),
            "cut_to_end_of_line should work in Normal mode"
        );

        // Verify text was cut from buffer
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "hello ",
            "Text from cursor to end should be removed"
        );

        // Verify yanked text is in buffer
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("world".to_string()),
            "Cut text should be in yank buffer"
        );
    }

    #[test]
    fn test_cut_to_end_of_line_at_end_of_line() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move cursor to end of line
        vm.set_cursor_position(LogicalPosition { line: 0, column: 5 })
            .unwrap();

        // Cut from cursor to end of line (should cut nothing)
        let result = vm.cut_to_end_of_line();
        assert!(result.is_ok(), "cut_to_end_of_line should work even at end");

        // Verify text unchanged
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "hello",
            "Text should be unchanged when at end"
        );
    }

    #[test]
    fn test_cut_to_end_of_line_whole_line() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("entire line").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move cursor to beginning of line
        vm.set_cursor_position(LogicalPosition { line: 0, column: 0 })
            .unwrap();

        // Cut from beginning to end
        let result = vm.cut_to_end_of_line();
        assert!(result.is_ok(), "cut_to_end_of_line should work");

        // Verify entire line was cut
        let request_text = vm.get_request_text();
        assert_eq!(request_text, "", "Entire line should be removed");

        // Verify yanked text
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("entire line".to_string()),
            "Entire line should be yanked"
        );
    }

    #[test]
    fn test_cut_to_end_of_line_blocked_in_insert_mode() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add test content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();
        // Stay in Insert mode

        // Try to cut (should be blocked)
        let result = vm.cut_to_end_of_line();
        assert!(result.is_ok(), "Method should return Ok but do nothing");

        // Verify text unchanged
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "hello world",
            "Text should be unchanged in Insert mode"
        );

        // Verify nothing was yanked
        let yanked = vm.get_yanked_text();
        assert!(yanked.is_none(), "Nothing should be yanked in Insert mode");
    }

    #[test]
    fn test_cut_to_end_of_line_with_multibyte_characters() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add content with multibyte characters
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("こんにちは world").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move cursor to position 5 (after "こんにちは")
        vm.set_cursor_position(LogicalPosition { line: 0, column: 5 })
            .unwrap();

        // Cut from cursor to end
        let result = vm.cut_to_end_of_line();
        assert!(
            result.is_ok(),
            "cut_to_end_of_line should work with multibyte chars"
        );

        // Verify correct text was cut
        let request_text = vm.get_request_text();
        assert_eq!(request_text, "こんにちは", "Japanese text should remain");

        // Verify yanked text
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some(" world".to_string()),
            "English part should be yanked"
        );
    }

    #[test]
    fn test_cut_current_line_in_normal_mode() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add multiple lines
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move to line 1 (middle line)
        vm.set_cursor_position(LogicalPosition { line: 1, column: 3 })
            .unwrap();

        // Cut current line
        let result = vm.cut_current_line();
        assert!(
            result.is_ok(),
            "cut_current_line should work in Normal mode"
        );

        // Verify line was removed and cursor moved appropriately
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "line 1\nline 3",
            "Middle line should be removed"
        );

        // Verify cursor moved to beginning of next line (now line 1)
        let cursor_pos = vm.get_cursor_position();
        assert_eq!(
            cursor_pos,
            LogicalPosition { line: 1, column: 0 },
            "Cursor should be at beginning of next line"
        );

        // Verify yanked text is in buffer with newline (Line type)
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("line 2\n".to_string()),
            "Cut line should be in yank buffer with newline"
        );
    }

    #[test]
    fn test_cut_current_line_last_line() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add multiple lines
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move to last line (line 2)
        vm.set_cursor_position(LogicalPosition { line: 2, column: 2 })
            .unwrap();

        // Cut current line
        let result = vm.cut_current_line();
        assert!(result.is_ok(), "cut_current_line should work on last line");

        // Verify last line was removed
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "line 1\nline 2",
            "Last line should be removed"
        );

        // Verify cursor moved to beginning of previous line (now last line)
        let cursor_pos = vm.get_cursor_position();
        assert_eq!(
            cursor_pos,
            LogicalPosition { line: 1, column: 0 },
            "Cursor should be at beginning of new last line"
        );

        // Verify yanked text
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("line 3\n".to_string()),
            "Cut line should be in yank buffer"
        );
    }

    #[test]
    fn test_cut_current_line_single_line() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add single line
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("only line").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move cursor to middle of line
        vm.set_cursor_position(LogicalPosition { line: 0, column: 3 })
            .unwrap();

        // Cut current line
        let result = vm.cut_current_line();
        assert!(
            result.is_ok(),
            "cut_current_line should work on single line"
        );

        // Verify line was removed, leaving empty buffer
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "",
            "Single line should be removed, leaving empty"
        );

        // Verify cursor at line 0, column 0
        let cursor_pos = vm.get_cursor_position();
        assert_eq!(
            cursor_pos,
            LogicalPosition { line: 0, column: 0 },
            "Cursor should be at origin after cutting only line"
        );

        // Verify yanked text
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("only line\n".to_string()),
            "Cut line should be in yank buffer"
        );
    }

    #[test]
    fn test_cut_current_line_blocked_in_insert_mode() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line 1\nline 2").unwrap();
        // Stay in Insert mode

        // Try to cut (should be blocked)
        let result = vm.cut_current_line();
        assert!(result.is_ok(), "Method should return Ok but do nothing");

        // Verify text unchanged
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "line 1\nline 2",
            "Text should be unchanged in Insert mode"
        );

        // Verify nothing was yanked
        let yanked = vm.get_yanked_text();
        assert!(yanked.is_none(), "Nothing should be yanked in Insert mode");
    }

    #[test]
    fn test_cut_current_line_with_multibyte_characters() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add content with multibyte characters
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("こんにちは\n世界\nHello").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Move to line 1 (Japanese line)
        vm.set_cursor_position(LogicalPosition { line: 1, column: 1 })
            .unwrap();

        // Cut current line
        let result = vm.cut_current_line();
        assert!(
            result.is_ok(),
            "cut_current_line should work with multibyte chars"
        );

        // Verify correct line was cut
        let request_text = vm.get_request_text();
        assert_eq!(
            request_text, "こんにちは\nHello",
            "Japanese line should be removed"
        );

        // Verify cursor moved to beginning of next line
        let cursor_pos = vm.get_cursor_position();
        assert_eq!(
            cursor_pos,
            LogicalPosition { line: 1, column: 0 },
            "Cursor should be at beginning of next line"
        );

        // Verify yanked text
        let yanked = vm.get_yanked_text();
        assert_eq!(
            yanked,
            Some("世界\n".to_string()),
            "Japanese text should be yanked"
        );
    }

    #[test]
    fn test_cut_current_line_yank_type_is_line() {
        let mut vm = ViewModel::new();

        // Start in Insert mode and add content
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("test line").unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        // Cut current line
        let result = vm.cut_current_line();
        assert!(result.is_ok(), "cut_current_line should work");

        // Verify yanked entry is Line type
        let yanked_entry = vm.get_yanked_entry();
        assert!(yanked_entry.is_some(), "Should have yanked entry");

        let entry = yanked_entry.unwrap();
        assert_eq!(
            entry.text, "test line\n",
            "Yanked text should include newline"
        );
        assert_eq!(entry.yank_type, YankType::Line, "Yank type should be Line");
    }
}
