//! # Buffer Operations
//!
//! Handles text insertion, deletion, and buffer content manipulation.

use crate::repl::events::{EditorMode, LogicalPosition, LogicalRange, Pane, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.current_pane != Pane::Request || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Get insertion position BEFORE making any changes
        let insertion_pos = self.panes[Pane::Request].buffer.cursor();

        let _event = self.panes[Pane::Request].buffer.insert_char(ch);
        // TODO: self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);

        // Sync display cursor with logical cursor
        self.sync_display_cursor_with_logical(Pane::Request)?;

        // BUGFIX: Ensure cursor is visible BEFORE calculating display coordinates
        // This ensures scrolling happens first, then we calculate coordinates with the correct scroll offset
        self.ensure_cursor_visible(Pane::Request);

        // Calculate display line for the insertion position AFTER potential scrolling
        let insertion_display_line = self.panes[Pane::Request]
            .display_cache
            .logical_to_display_position(insertion_pos.line, insertion_pos.column)
            .map(|(display_line, _)| display_line)
            .unwrap_or(0);

        // BUGFIX: Adjust start_line to be relative to current scroll offset
        // The terminal renderer expects viewport-relative coordinates, not absolute display lines
        let (vertical_scroll_offset, _) = self.get_scroll_offset(Pane::Request);
        let viewport_start_line = insertion_display_line.saturating_sub(vertical_scroll_offset);

        // Use partial redraw from the viewport-relative line where the character was inserted
        self.emit_view_event([ViewEvent::PartialPaneRedrawRequired {
            pane: Pane::Request,
            start_line: viewport_start_line,
        }]);

        Ok(())
    }

    /// Insert text at current cursor position
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.current_pane != Pane::Request || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        // Get insertion position BEFORE making any changes
        let insertion_pos = self.panes[Pane::Request].buffer.cursor();

        let _event = self.panes[Pane::Request].buffer.insert_text(text);
        // TODO: self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);

        // Sync display cursor with logical cursor
        self.sync_display_cursor_with_logical(Pane::Request)?;

        // BUGFIX: Ensure cursor is visible BEFORE calculating display coordinates
        // This ensures scrolling happens first, then we calculate coordinates with the correct scroll offset
        self.ensure_cursor_visible(Pane::Request);

        // Calculate display line for the insertion position AFTER potential scrolling
        let insertion_display_line = self.panes[Pane::Request]
            .display_cache
            .logical_to_display_position(insertion_pos.line, insertion_pos.column)
            .map(|(display_line, _)| display_line)
            .unwrap_or(0);

        // BUGFIX: Adjust start_line to be relative to current scroll offset
        // The terminal renderer expects viewport-relative coordinates, not absolute display lines
        let (vertical_scroll_offset, _) = self.get_scroll_offset(Pane::Request);
        let viewport_start_line = insertion_display_line.saturating_sub(vertical_scroll_offset);

        // Use partial redraw from the viewport-relative line where the text was inserted
        self.emit_view_event([ViewEvent::PartialPaneRedrawRequired {
            pane: Pane::Request,
            start_line: viewport_start_line,
        }]);

        // Update cursor position after text insertion
        self.emit_view_event([
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Request,
            },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        Ok(())
    }

    /// Delete character before cursor
    pub fn delete_char_before_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.current_pane != Pane::Request || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pane = self.current_pane;
        let current_pos = self.panes[current_pane].buffer.cursor();

        if current_pos.column > 0 {
            // Delete character in current line
            let delete_pos = LogicalPosition::new(current_pos.line, current_pos.column - 1);
            let range = LogicalRange::single_char(delete_pos);

            if let Some(_event) = self.panes[current_pane]
                .buffer
                .content_mut()
                .delete_range(current_pane, range)
            {
                // Move cursor back
                self.panes[current_pane].buffer.set_cursor(delete_pos);

                // Rebuild display cache after content change
                let content_width = self.get_content_width();
                self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                // Sync display cursors to update cursor position
                self.sync_display_cursors();

                self.emit_view_event([
                    ViewEvent::PaneRedrawRequired { pane: current_pane },
                    ViewEvent::CursorUpdateRequired { pane: current_pane },
                ]);
            }
        } else if current_pos.line > 0 {
            // Check if current line is blank (empty)
            let current_line_length = self.panes[current_pane]
                .buffer
                .content()
                .line_length(current_pos.line);

            if current_line_length == 0 {
                // Current line is blank - delete entire line and move to end of previous line
                let prev_line_length = self.panes[current_pane]
                    .buffer
                    .content()
                    .line_length(current_pos.line - 1);
                let new_cursor = LogicalPosition::new(current_pos.line - 1, prev_line_length);

                // Delete the blank line by removing the newline at the end of the previous line
                let range = LogicalRange::new(
                    LogicalPosition::new(current_pos.line - 1, prev_line_length),
                    LogicalPosition::new(current_pos.line, 0),
                );

                if let Some(_event) = self.panes[current_pane]
                    .buffer
                    .content_mut()
                    .delete_range(current_pane, range)
                {
                    self.panes[current_pane].buffer.set_cursor(new_cursor);

                    // Rebuild display cache after content change
                    let content_width = self.get_content_width();
                    self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursors to update cursor position
                    self.sync_display_cursors();

                    self.emit_view_event([ViewEvent::PaneRedrawRequired { pane: current_pane }]);
                    self.emit_view_event([ViewEvent::CursorUpdateRequired { pane: current_pane }]);
                }
            } else {
                // Join with previous line (existing behavior)
                let prev_line_length = self.panes[current_pane]
                    .buffer
                    .content()
                    .line_length(current_pos.line - 1);
                let new_cursor = LogicalPosition::new(current_pos.line - 1, prev_line_length);

                // Delete the newline between the previous and current line
                let range = LogicalRange::new(
                    LogicalPosition::new(current_pos.line - 1, prev_line_length),
                    LogicalPosition::new(current_pos.line, 0),
                );

                if self.panes[current_pane]
                    .buffer
                    .content_mut()
                    .delete_range(current_pane, range)
                    .is_some()
                {
                    // Lines are already joined by delete_range, no need to insert content again
                    // The delete_range removed only the newline, keeping both line contents intact
                    self.panes[current_pane].buffer.set_cursor(new_cursor);

                    // Rebuild display cache after content change
                    let content_width = self.get_content_width();
                    self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursors to update cursor position
                    self.sync_display_cursors();

                    self.emit_view_event([ViewEvent::PaneRedrawRequired { pane: current_pane }]);
                    self.emit_view_event([ViewEvent::CursorUpdateRequired { pane: current_pane }]);
                }
            }
        }

        Ok(())
    }

    /// Delete character after cursor or empty line
    pub fn delete_char_after_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.current_pane != Pane::Request || self.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pane = self.current_pane;
        let current_pos = self.panes[current_pane].buffer.cursor();
        let current_line_length = self.panes[current_pane]
            .buffer
            .content()
            .line_length(current_pos.line);
        let total_lines = self.panes[current_pane].buffer.content().line_count();

        if current_pos.column < current_line_length {
            // Delete character in current line
            let range = LogicalRange::single_char(current_pos);

            if let Some(_event) = self.panes[current_pane]
                .buffer
                .content_mut()
                .delete_range(current_pane, range)
            {
                // Rebuild request display cache after content change
                let content_width = self.get_content_width();
                self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                // Sync display cursors to ensure cursor position is correct
                self.sync_display_cursors();

                self.emit_view_event([
                    ViewEvent::PaneRedrawRequired { pane: current_pane },
                    ViewEvent::CursorUpdateRequired { pane: current_pane },
                ]);
            }
        } else if current_line_length == 0 && current_pos.line > 0 {
            // Current line is empty and not the first line - delete it
            let prev_line_length = self.panes[current_pane]
                .buffer
                .content()
                .line_length(current_pos.line - 1);
            let new_cursor = LogicalPosition::new(current_pos.line - 1, prev_line_length);

            // Delete the empty line by removing the newline at the end of the previous line
            let range = LogicalRange::new(
                LogicalPosition::new(current_pos.line - 1, prev_line_length),
                LogicalPosition::new(current_pos.line, 0),
            );

            if let Some(_event) = self.panes[current_pane]
                .buffer
                .content_mut()
                .delete_range(current_pane, range)
            {
                self.panes[current_pane].buffer.set_cursor(new_cursor);

                // Rebuild display cache after content change
                let content_width = self.get_content_width();
                self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                // Sync display cursors to update cursor position
                self.sync_display_cursors();

                self.emit_view_event([
                    ViewEvent::PaneRedrawRequired { pane: current_pane },
                    ViewEvent::CursorUpdateRequired { pane: current_pane },
                ]);
            }
        } else if current_pos.line + 1 < total_lines {
            // At end of current line, check if next line exists
            let next_line_length = self.panes[current_pane]
                .buffer
                .content()
                .line_length(current_pos.line + 1);

            if next_line_length == 0 {
                // Next line is empty - delete it by removing the newline that creates it
                let range = LogicalRange::new(
                    LogicalPosition::new(current_pos.line, current_line_length),
                    LogicalPosition::new(current_pos.line + 1, 0),
                );

                if let Some(_event) = self.panes[current_pane]
                    .buffer
                    .content_mut()
                    .delete_range(current_pane, range)
                {
                    // Cursor stays at end of current line
                    // Rebuild display cache after content change
                    let content_width = self.get_content_width();
                    self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursors to ensure cursor position is correct
                    self.sync_display_cursors();

                    self.emit_view_event([
                        ViewEvent::PaneRedrawRequired { pane: current_pane },
                        ViewEvent::CursorUpdateRequired { pane: current_pane },
                    ]);
                }
            } else {
                // Join current line with next line by removing the newline between them
                let range = LogicalRange::new(
                    LogicalPosition::new(current_pos.line, current_line_length),
                    LogicalPosition::new(current_pos.line + 1, 0),
                );

                if let Some(_event) = self.panes[current_pane]
                    .buffer
                    .content_mut()
                    .delete_range(current_pane, range)
                {
                    // Cursor stays at end of current line (now joined with next line content)
                    // Rebuild display cache after content change
                    let content_width = self.get_content_width();
                    self.panes[current_pane].build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursors to ensure cursor position is correct
                    self.sync_display_cursors();

                    self.emit_view_event([
                        ViewEvent::PaneRedrawRequired { pane: current_pane },
                        ViewEvent::CursorUpdateRequired { pane: current_pane },
                    ]);
                }
            }
        }

        Ok(())
    }
}
