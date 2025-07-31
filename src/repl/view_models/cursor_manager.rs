//! # Cursor Management
//!
//! Handles all cursor movement and positioning logic using semantic operations from PaneManager.
//! This module provides high-level cursor operations that work with the current/other area abstraction.

use crate::repl::events::LogicalPosition;
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get current logical cursor position for the active area
    pub fn get_cursor_position(&self) -> LogicalPosition {
        self.pane_manager.get_current_cursor_position()
    }

    /// Get current display cursor position for the active area
    pub fn get_display_cursor_position(&self) -> (usize, usize) {
        self.pane_manager.get_current_display_cursor()
    }

    /// Move cursor left in current area
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_left();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor right in current area
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_right();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor up in current area
    pub fn move_cursor_up(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_up();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor down in current area
    pub fn move_cursor_down(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_down();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_end_of_line();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_start_of_line();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to start of document
    pub fn move_cursor_to_document_start(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_document_start();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to end of document
    pub fn move_cursor_to_document_end(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_document_end();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Set cursor position in current area
    pub fn set_cursor_position(&mut self, position: LogicalPosition) -> Result<()> {
        let events = self.pane_manager.set_current_cursor_position(position);
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to next word in current area
    pub fn move_cursor_to_next_word(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_next_word();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to previous word in current area
    pub fn move_cursor_to_previous_word(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_previous_word();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to end of word in current area
    pub fn move_cursor_to_end_of_word(&mut self) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_end_of_word();
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    /// Move cursor to specific line number (1-based)
    pub fn move_cursor_to_line(&mut self, line_number: usize) -> Result<()> {
        let events = self.pane_manager.move_cursor_to_line(line_number);
        if !events.is_empty() {
            let content_width = self.get_content_width();
            let visibility_events = self
                .pane_manager
                .ensure_current_cursor_visible(content_width);
            let mut all_events = events;
            all_events.extend(visibility_events);
            self.emit_view_event(all_events);
        }
        Ok(())
    }

    // Scrolling methods are implemented elsewhere - avoiding duplication
}
