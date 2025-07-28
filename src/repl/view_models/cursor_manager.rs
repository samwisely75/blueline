//! # Cursor Management
//!
//! Handles all cursor movement and positioning logic including display cursor synchronization,
//! scrolling, and coordinate transformations between logical and display positions.

use crate::repl::events::{LogicalPosition, Pane, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get current logical cursor position for the active pane
    pub fn get_cursor_position(&self) -> LogicalPosition {
        match self.editor.current_pane() {
            Pane::Request => self.request_buffer.cursor(),
            Pane::Response => self.response_buffer.cursor(),
        }
    }

    /// Move cursor left in current pane (display coordinate based)
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_left: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut moved = false;

        // Check if we can move left within current display line
        if current_display_pos.1 > 0 {
            let new_display_pos = (current_display_pos.0, current_display_pos.1 - 1);
            tracing::debug!(
                "move_cursor_left: moving within line to {:?}",
                new_display_pos
            );
            self.set_display_cursor(current_pane, new_display_pos)?;
            self.ensure_cursor_visible(current_pane);
            moved = true;
        } else if current_display_pos.0 > 0 {
            // Move to end of previous display line
            let prev_display_line = current_display_pos.0 - 1;
            if let Some(prev_line) = display_cache.get_display_line(prev_display_line) {
                let new_col = prev_line.content.chars().count();
                let new_display_pos = (prev_display_line, new_col);
                tracing::debug!(
                    "move_cursor_left: moving to end of previous line {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            }
        } else {
            tracing::debug!("move_cursor_left: already at beginning, no movement");
        }

        // Only emit events if we actually moved
        if moved {
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);
        }

        Ok(())
    }

    /// Move cursor right in current pane (display coordinate based)
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_right: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut moved = false;

        // Get current display line info
        if let Some(current_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_length = current_line.content.chars().count();

            // Check if we can move right within current display line
            if current_display_pos.1 < line_length {
                let new_display_pos = (current_display_pos.0, current_display_pos.1 + 1);
                tracing::debug!(
                    "move_cursor_right: moving within line to {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            } else if current_display_pos.0 + 1 < display_cache.display_line_count() {
                // Move to beginning of next display line
                let new_display_pos = (current_display_pos.0 + 1, 0);
                tracing::debug!(
                    "move_cursor_right: moving to next line {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            } else {
                tracing::debug!("move_cursor_right: already at end, no movement");
            }
        } else {
            tracing::debug!("move_cursor_right: invalid display line, no movement");
        }

        // Only emit events if we actually moved
        if moved {
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);
        }

        Ok(())
    }

    /// Move cursor up in current pane (display line based)
    pub fn move_cursor_up(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_up: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        if let Some(new_pos) = display_cache.move_up(current_display_pos.0, current_display_pos.1) {
            self.set_display_cursor(current_pane, new_pos)?;
            self.ensure_cursor_visible(current_pane);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);
        } else {
            tracing::debug!("move_cursor_up: already at top, no movement");
        }

        Ok(())
    }

    /// Move cursor down in current pane (display line based)
    pub fn move_cursor_down(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_down: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        if let Some(new_pos) = display_cache.move_down(current_display_pos.0, current_display_pos.1)
        {
            self.set_display_cursor(current_pane, new_pos)?;
            self.ensure_cursor_visible(current_pane);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);
        } else {
            tracing::debug!("move_cursor_down: already at bottom, no movement");
        }

        Ok(())
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        if let Some(current_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_length = current_line.content.chars().count();
            let new_pos = (current_display_pos.0, line_length);
            self.set_display_cursor(current_pane, new_pos)?;
            self.ensure_cursor_visible(current_pane);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);
        }

        Ok(())
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let new_pos = (current_display_pos.0, 0);
        self.set_display_cursor(current_pane, new_pos)?;
        self.ensure_cursor_visible(current_pane);
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        Ok(())
    }

    /// Move cursor to start of document (gg command)
    pub fn move_cursor_to_document_start(&mut self) -> Result<()> {
        // Set logical cursor to (0, 0) - first line, first column
        let start_position = LogicalPosition::new(0, 0);
        self.set_cursor_position(start_position)?;

        Ok(())
    }

    /// Move cursor to end of document (G command)
    pub fn move_cursor_to_document_end(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Get the content to find the last logical position
        let content = match current_pane {
            Pane::Request => self.request_buffer.content(),
            Pane::Response => self.response_buffer.content(),
        };

        let lines = content.lines();
        let last_line_index = lines.len().saturating_sub(1);
        let last_line_content = lines.get(last_line_index).map_or("", |s| s.as_str());
        let last_column = last_line_content.chars().count();

        // Set logical cursor to last position
        let end_position = LogicalPosition::new(last_line_index, last_column);
        self.set_cursor_position(end_position)?;

        Ok(())
    }

    /// Set cursor to specific logical position
    pub fn set_cursor_position(&mut self, position: LogicalPosition) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Update logical cursor in appropriate buffer
        match current_pane {
            Pane::Request => {
                let clamped_position = self.request_buffer.content().clamp_position(position);
                self.request_buffer.set_cursor(clamped_position);
            }
            Pane::Response => {
                let clamped_position = self.response_buffer.content().clamp_position(position);
                self.response_buffer.set_cursor(clamped_position);
            }
        }

        // Sync display cursor
        self.sync_display_cursor_with_logical(current_pane)?;
        self.ensure_cursor_visible(current_pane);
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        Ok(())
    }

    /// Synchronize display cursors for both panes
    pub(super) fn sync_display_cursors(&mut self) {
        // Update request display cursor
        let request_logical = self.request_buffer.cursor();
        if let Some(display_pos) = self
            .request_display_cache
            .logical_to_display_position(request_logical.line, request_logical.column)
        {
            self.request_display_cursor = display_pos;
        }

        // Update response display cursor
        let response_logical = self.response_buffer.cursor();
        if let Some(display_pos) = self
            .response_display_cache
            .logical_to_display_position(response_logical.line, response_logical.column)
        {
            self.response_display_cursor = display_pos;
        }
    }

    /// Get display cursor position for a pane
    pub(super) fn get_display_cursor(&self, pane: Pane) -> (usize, usize) {
        match pane {
            Pane::Request => self.request_display_cursor,
            Pane::Response => self.response_display_cursor,
        }
    }

    /// Set display cursor position for a pane
    pub(super) fn set_display_cursor(
        &mut self,
        pane: Pane,
        position: (usize, usize),
    ) -> Result<()> {
        match pane {
            Pane::Request => {
                self.request_display_cursor = position;
                // Convert back to logical position and update buffer
                if let Some(logical_pos) = self
                    .request_display_cache
                    .display_to_logical_position(position.0, position.1)
                {
                    let logical_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
                    self.request_buffer.set_cursor(logical_position);
                }
            }
            Pane::Response => {
                self.response_display_cursor = position;
                // Convert back to logical position and update buffer
                if let Some(logical_pos) = self
                    .response_display_cache
                    .display_to_logical_position(position.0, position.1)
                {
                    let logical_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
                    self.response_buffer.set_cursor(logical_position);
                }
            }
        }
        Ok(())
    }

    /// Synchronize display cursor with logical cursor position
    pub(super) fn sync_display_cursor_with_logical(&mut self, pane: Pane) -> Result<()> {
        let logical_pos = match pane {
            Pane::Request => self.request_buffer.cursor(),
            Pane::Response => self.response_buffer.cursor(),
        };

        let display_cache = self.get_display_cache(pane);

        if let Some(display_pos) =
            display_cache.logical_to_display_position(logical_pos.line, logical_pos.column)
        {
            match pane {
                Pane::Request => self.request_display_cursor = display_pos,
                Pane::Response => self.response_display_cursor = display_pos,
            }
        }

        Ok(())
    }

    /// Ensure cursor is visible within the viewport (handles scrolling)
    pub(super) fn ensure_cursor_visible(&mut self, pane: Pane) {
        let display_pos = self.get_display_cursor(pane);
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(pane);
        let pane_height = self.get_pane_display_height(pane);
        let content_width = self.get_content_width();

        let mut new_vertical_offset = vertical_offset;
        let mut new_horizontal_offset = horizontal_offset;

        // Vertical scrolling
        if display_pos.0 < vertical_offset {
            new_vertical_offset = display_pos.0;
        } else if display_pos.0 >= vertical_offset + pane_height {
            new_vertical_offset = display_pos.0.saturating_sub(pane_height - 1);
        }

        // Horizontal scrolling
        if display_pos.1 < horizontal_offset {
            new_horizontal_offset = display_pos.1;
        } else if display_pos.1 >= horizontal_offset + content_width {
            new_horizontal_offset = display_pos.1.saturating_sub(content_width - 1);
        }

        // Update scroll offset if changed
        if new_vertical_offset != vertical_offset || new_horizontal_offset != horizontal_offset {
            let old_offset = (vertical_offset, horizontal_offset);
            self.set_scroll_offset(pane, (new_vertical_offset, new_horizontal_offset));

            // Emit scroll changed event
            self.emit_view_event(ViewEvent::ScrollChanged {
                pane,
                old_offset: old_offset.0,
                new_offset: new_vertical_offset,
            });
        }
    }

    /// Get scroll offset for a pane
    pub(super) fn get_scroll_offset(&self, pane: Pane) -> (usize, usize) {
        match pane {
            Pane::Request => self.request_scroll_offset,
            Pane::Response => self.response_scroll_offset,
        }
    }

    /// Set scroll offset for a pane
    pub(super) fn set_scroll_offset(&mut self, pane: Pane, offset: (usize, usize)) {
        match pane {
            Pane::Request => self.request_scroll_offset = offset,
            Pane::Response => self.response_scroll_offset = offset,
        }
    }

    /// Get display height for a pane
    fn get_pane_display_height(&self, pane: Pane) -> usize {
        match pane {
            Pane::Request => self.request_pane_height as usize,
            Pane::Response => {
                if self.response.status_code().is_some() {
                    (self.terminal_height - self.request_pane_height - 2) as usize
                // -2 for separator and status
                } else {
                    0
                }
            }
        }
    }
}
