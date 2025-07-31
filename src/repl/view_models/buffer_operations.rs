//! # Buffer Operations
//!
//! Handles text insertion, deletion, and buffer content manipulation.

use crate::repl::events::{EditorMode, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in Request pane and insert mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Use semantic insertion from PaneManager
        let events = self.pane_manager.insert_char_in_request(ch);

        // Ensure cursor is visible after insertion
        let content_width = self.get_content_width();
        let visibility_events = self
            .pane_manager
            .ensure_current_cursor_visible(content_width);

        // Emit all events
        let mut all_events = events;
        all_events.extend(visibility_events);
        all_events.push(ViewEvent::ActiveCursorUpdateRequired);
        all_events.push(ViewEvent::PositionIndicatorUpdateRequired);

        self.emit_view_event(all_events);

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
        // Only allow deletion in Request pane and insert mode
        if !self.is_in_request_pane() || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Use semantic deletion from PaneManager
        let events = self.pane_manager.delete_char_before_cursor_in_request();

        // Ensure cursor is visible after deletion
        let content_width = self.get_content_width();
        let visibility_events = self
            .pane_manager
            .ensure_current_cursor_visible(content_width);

        // Emit all events
        let mut all_events = events;
        all_events.extend(visibility_events);
        all_events.push(ViewEvent::ActiveCursorUpdateRequired);
        all_events.push(ViewEvent::PositionIndicatorUpdateRequired);

        self.emit_view_event(all_events);

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

        // Ensure cursor is visible after deletion
        let content_width = self.get_content_width();
        let visibility_events = self
            .pane_manager
            .ensure_current_cursor_visible(content_width);

        // Emit all events
        let mut all_events = events;
        all_events.extend(visibility_events);
        all_events.push(ViewEvent::ActiveCursorUpdateRequired);
        all_events.push(ViewEvent::PositionIndicatorUpdateRequired);

        self.emit_view_event(all_events);

        Ok(())
    }
}
