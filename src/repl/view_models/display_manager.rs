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
        &self.panes[pane].display_cache
    }

    /// Get display lines for rendering a specific pane
    pub fn get_display_lines_for_rendering(
        &self,
        pane: Pane,
        start_row: usize,
        row_count: usize,
    ) -> Vec<Option<DisplayLineData>> {
        let display_cache = self.get_display_cache(pane);
        let (vertical_scroll_offset, horizontal_scroll_offset) = self.get_scroll_offset(pane);
        let content_width = self.get_content_width();
        let mut result = Vec::new();

        for row in 0..row_count {
            let display_line_idx = vertical_scroll_offset + start_row + row;

            if let Some(display_line) = display_cache.get_display_line(display_line_idx) {
                // Apply horizontal scrolling to content
                let visible_content = if horizontal_scroll_offset < display_line.content.len() {
                    let end_pos =
                        (horizontal_scroll_offset + content_width).min(display_line.content.len());
                    display_line.content[horizontal_scroll_offset..end_pos].to_string()
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
        let display_pos = self.get_display_cursor(pane);
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(pane);

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
            Pane::Request => self.get_request_text(),
            Pane::Response => self.get_response_text(),
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

    /// Get content width (terminal width minus line numbers and padding)
    /// Note: This is a simplified calculation. In practice, each pane may have different line number widths.
    pub(super) fn get_content_width(&self) -> usize {
        // Use current pane's line number width
        let current_pane = self.current_pane;
        let line_num_width = self.get_line_number_width(current_pane);

        (self.terminal_dimensions.0 as usize).saturating_sub(line_num_width + 1)
    }

    /// Set word wrap enabled/disabled and rebuild display caches
    pub fn set_wrap_enabled(&mut self, enabled: bool) -> Result<(), anyhow::Error> {
        if self.wrap_enabled != enabled {
            self.wrap_enabled = enabled;
            self.rebuild_display_caches()?;
            self.emit_view_event([ViewEvent::FullRedrawRequired]);
        }
        Ok(())
    }

    /// Rebuild display caches for both panes
    fn rebuild_display_caches(&mut self) -> Result<(), anyhow::Error> {
        let content_width = self.get_content_width();

        // Rebuild display caches using PaneState encapsulation
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);
        self.panes[Pane::Response].build_display_cache(content_width, self.wrap_enabled);

        // Synchronize display cursors after rebuilding caches to handle coordinate system changes
        // This is critical when wrap mode changes as cursor positions may be invalid
        self.sync_display_cursors();

        // BUGFIX: Ensure cursor remains visible after wrap mode toggle instead of resetting to (0,0)
        // Without this fix, toggling wrap mode resets scroll to top while cursor position indicator
        // shows the old position (e.g., "RESPONSE 56:1"), causing navigation to behave incorrectly
        // Using ensure_cursor_visible() maintains proper cursor-scroll synchronization
        self.ensure_cursor_visible(Pane::Request);
        self.ensure_cursor_visible(Pane::Response);

        Ok(())
    }
}
