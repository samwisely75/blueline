//! # Display Management
//!
//! Handles display cache management, word wrapping, and display coordinate calculations.
//! This module coordinates between logical content and display representation.

use crate::repl::events::{Pane, ViewEvent};
use crate::repl::models::DisplayCache;
use crate::repl::view_models::core::{DisplayLineData, ViewModel};

/// Minimum width for line number column as specified in requirements
const MIN_LINE_NUMBER_WIDTH: usize = 3;

impl ViewModel {
    /// Get display cache for a specific pane
    pub(super) fn get_display_cache(&self, pane: Pane) -> &DisplayCache {
        self.pane_manager.get_display_cache(pane)
    }

    /// Get display lines for rendering a specific pane
    pub fn get_display_lines_for_rendering(
        &self,
        pane: Pane,
        start_row: usize,
        row_count: usize,
    ) -> Vec<Option<DisplayLineData>> {
        let display_cache = self.get_display_cache(pane);
        let (vertical_scroll_offset, horizontal_scroll_offset) =
            if pane == self.pane_manager.current_pane_type() {
                self.pane_manager.get_current_scroll_offset()
            } else {
                // For non-current pane, access scroll offset directly (this should be made semantic later)
                (0, 0) // Simplified for now
            };
        let content_width = self.get_content_width();
        let mut result = Vec::new();

        for row in 0..row_count {
            let display_line_idx = vertical_scroll_offset + start_row + row;

            if let Some(display_line) = display_cache.get_display_line(display_line_idx) {
                // Apply horizontal scrolling to content
                let content = display_line.content();
                let visible_content = if horizontal_scroll_offset < content.len() {
                    let end_pos = (horizontal_scroll_offset + content_width).min(content.len());
                    content[horizontal_scroll_offset..end_pos].to_string()
                } else {
                    String::new()
                };

                // Show logical line number only for first segment of wrapped lines
                let line_number = if display_line.is_continuation {
                    None
                } else {
                    Some(display_line.logical_line + 1) // 1-based line numbers
                };
                // Fourth parameter provides logical start column for accurate visual selection in wrapped lines
                // Fifth parameter provides logical line number for all lines (including continuations)
                result.push(Some((
                    visible_content,
                    line_number,
                    display_line.is_continuation,
                    display_line.logical_start_col + horizontal_scroll_offset,
                    display_line.logical_line, // Always provide logical line number
                )));
            } else {
                // Beyond content - show tilde (false indicates this is beyond content, not continuation)
                result.push(None);
            }
        }

        result
    }

    /// Get cursor position for rendering
    pub fn get_cursor_for_rendering(&self, pane: Pane) -> (usize, usize) {
        let display_pos = if pane == self.pane_manager.current_pane_type() {
            self.pane_manager.get_current_display_cursor()
        } else {
            // For non-current pane, simplified for now
            (0, 0)
        };
        let (vertical_offset, horizontal_offset) = if pane == self.pane_manager.current_pane_type()
        {
            self.pane_manager.get_current_scroll_offset()
        } else {
            // For non-current pane, simplified for now
            (0, 0)
        };

        // Calculate screen-relative position
        let screen_row = display_pos.0.saturating_sub(vertical_offset);
        let screen_col = display_pos.1.saturating_sub(horizontal_offset);

        (screen_row, screen_col)
    }

    /// Get line number width for a pane
    /// BUGFIX: Calculate dynamic line number width based on document size
    /// Without this dynamic calculation, cursor positioning becomes invalid for large documents
    /// (e.g., jumping to line 1547 with hardcoded width=3 causes cursor to appear next to "7" of "1547")
    pub fn get_line_number_width(&self, pane: Pane) -> usize {
        let content = match pane {
            Pane::Request => self.pane_manager.get_request_text(),
            Pane::Response => self.pane_manager.get_response_text(),
        };

        let line_count = if content.is_empty() {
            1 // At least show line 1 even for empty content
        } else {
            content.lines().count().max(1)
        };

        // Calculate width needed for the largest line number to prevent cursor positioning bugs
        let width = line_count.to_string().len();

        // Minimum width as specified in the requirements (never smaller than 3)
        width.max(MIN_LINE_NUMBER_WIDTH)
    }

    // get_content_width method moved to core.rs to avoid duplication

    /// Set word wrap enabled/disabled and rebuild display caches
    pub fn set_wrap_enabled(&mut self, enabled: bool) -> Result<(), anyhow::Error> {
        if self.pane_manager.is_wrap_enabled() != enabled {
            self.pane_manager.set_wrap_enabled(enabled);
            let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
            let mut events = vec![ViewEvent::FullRedrawRequired];
            events.extend(visibility_events);
            self.emit_view_event(events)?;
        }
        Ok(())
    }
}
