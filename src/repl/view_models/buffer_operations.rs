//! # Buffer Operations
//!
//! Handles text insertion, deletion, and buffer content manipulation.

use crate::repl::events::{EditorMode, LogicalPosition, LogicalRange, Pane, ViewEvent};
use crate::repl::models::build_display_cache;
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let _event = self.request_buffer.insert_char(ch);
        // TODO: self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        let request_lines = self.request_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&request_lines, content_width, self.wrap_enabled) {
            self.request_display_cache = cache;
        }

        // Sync display cursor with logical cursor
        self.sync_display_cursor_with_logical(Pane::Request)?;

        // Ensure cursor is visible after insertion (enables auto-horizontal scroll)
        self.ensure_cursor_visible(Pane::Request);

        // Determine if we need full pane redraw or just partial
        let cursor_pos = self.request_buffer.cursor();
        let display_line = self
            .request_display_cache
            .logical_to_display_position(cursor_pos.line, cursor_pos.column)
            .map(|(display_line, _)| display_line)
            .unwrap_or(0);

        // Use partial redraw from current line to bottom
        self.emit_view_event(ViewEvent::PartialPaneRedrawRequired {
            pane: Pane::Request,
            start_line: display_line,
        });

        Ok(())
    }

    /// Insert text at current cursor position
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let _event = self.request_buffer.insert_text(text);
        // TODO: self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        let request_lines = self.request_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&request_lines, content_width, self.wrap_enabled) {
            self.request_display_cache = cache;
        }

        // Sync display cursor with logical cursor
        self.sync_display_cursor_with_logical(Pane::Request)?;

        // Determine if we need full pane redraw or just partial
        let cursor_pos = self.request_buffer.cursor();
        let display_line = self
            .request_display_cache
            .logical_to_display_position(cursor_pos.line, cursor_pos.column)
            .map(|(display_line, _)| display_line)
            .unwrap_or(0);

        // Use partial redraw from current line to bottom
        self.emit_view_event(ViewEvent::PartialPaneRedrawRequired {
            pane: Pane::Request,
            start_line: display_line,
        });

        // Ensure cursor is visible after content is redrawn (prevents ghost cursor race condition)
        self.ensure_cursor_visible(Pane::Request);

        // Update cursor position after text insertion
        self.emit_view_event(ViewEvent::CursorUpdateRequired {
            pane: Pane::Request,
        });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        Ok(())
    }

    /// Delete character before cursor
    pub fn delete_char_before_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pos = self.request_buffer.cursor();

        if current_pos.column > 0 {
            // Delete character in current line
            let delete_pos = LogicalPosition::new(current_pos.line, current_pos.column - 1);
            let range = LogicalRange::single_char(delete_pos);

            if let Some(_event) = self
                .request_buffer
                .content_mut()
                .delete_range(Pane::Request, range)
            {
                // Move cursor back
                self.request_buffer.set_cursor(delete_pos);

                // Rebuild request display cache after content change
                let content_width = self.get_content_width();
                let request_lines = self.request_buffer.content().lines().to_vec();
                if let Ok(cache) =
                    build_display_cache(&request_lines, content_width, self.wrap_enabled)
                {
                    self.request_display_cache = cache;
                }

                // Sync display cursors to update cursor position
                self.sync_display_cursors();

                self.emit_view_event(ViewEvent::PaneRedrawRequired {
                    pane: Pane::Request,
                });
                self.emit_view_event(ViewEvent::CursorUpdateRequired {
                    pane: Pane::Request,
                });
            }
        } else if current_pos.line > 0 {
            // Join with previous line
            let prev_line_length = self
                .request_buffer
                .content()
                .line_length(current_pos.line - 1);
            let new_cursor = LogicalPosition::new(current_pos.line - 1, prev_line_length);

            // Get current line content
            if let Some(current_line) = self.request_buffer.content().get_line(current_pos.line) {
                let current_line_content = current_line.clone();

                // Delete current line and append to previous
                let range = LogicalRange::new(
                    LogicalPosition::new(current_pos.line - 1, prev_line_length),
                    LogicalPosition::new(current_pos.line + 1, 0),
                );

                if self
                    .request_buffer
                    .content_mut()
                    .delete_range(Pane::Request, range)
                    .is_some()
                {
                    // Insert the content at the end of previous line
                    self.request_buffer.content_mut().insert_text(
                        Pane::Request,
                        new_cursor,
                        &current_line_content,
                    );

                    self.request_buffer.set_cursor(new_cursor);

                    // Rebuild request display cache after content change
                    let content_width = self.get_content_width();
                    let request_lines = self.request_buffer.content().lines().to_vec();
                    if let Ok(cache) =
                        build_display_cache(&request_lines, content_width, self.wrap_enabled)
                    {
                        self.request_display_cache = cache;
                    }

                    // Sync display cursors to update cursor position
                    self.sync_display_cursors();

                    self.emit_view_event(ViewEvent::PaneRedrawRequired {
                        pane: Pane::Request,
                    });
                    self.emit_view_event(ViewEvent::CursorUpdateRequired {
                        pane: Pane::Request,
                    });
                }
            }
        }

        Ok(())
    }

    /// Delete character after cursor
    pub fn delete_char_after_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pos = self.request_buffer.cursor();
        let current_line_length = self.request_buffer.content().line_length(current_pos.line);

        if current_pos.column < current_line_length {
            // Delete character in current line
            let range = LogicalRange::single_char(current_pos);

            if let Some(_event) = self
                .request_buffer
                .content_mut()
                .delete_range(Pane::Request, range)
            {
                // Rebuild request display cache after content change
                let content_width = self.get_content_width();
                let request_lines = self.request_buffer.content().lines().to_vec();
                if let Ok(cache) =
                    build_display_cache(&request_lines, content_width, self.wrap_enabled)
                {
                    self.request_display_cache = cache;
                }

                // Sync display cursors to ensure cursor position is correct
                self.sync_display_cursors();

                self.emit_view_event(ViewEvent::PaneRedrawRequired {
                    pane: Pane::Request,
                });
                self.emit_view_event(ViewEvent::CursorUpdateRequired {
                    pane: Pane::Request,
                });
            }
        }
        // Note: We don't handle joining with next line for simplicity
        // That would be a more complex operation

        Ok(())
    }
}
