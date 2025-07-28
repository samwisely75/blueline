//! # Display Management
//!
//! Handles display cache management, word wrapping, and display coordinate calculations.
//! This module coordinates between logical content and display representation.

use crate::repl::events::Pane;
use crate::repl::models::DisplayCache;
use crate::repl::view_models::core::{DisplayLineData, ViewModel};

impl ViewModel {
    /// Get display cache for a specific pane
    pub(super) fn get_display_cache(&self, pane: Pane) -> &DisplayCache {
        match pane {
            Pane::Request => &self.request_display_cache,
            Pane::Response => &self.response_display_cache,
        }
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
                // Third parameter indicates if this is a continuation line (true) or beyond content (false)
                result.push(Some((
                    visible_content,
                    line_number,
                    display_line.is_continuation,
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
    pub fn get_line_number_width(&self, _pane: Pane) -> usize {
        3 // Minimum width for now
    }

    /// Get content width (terminal width minus line numbers and padding)
    pub(super) fn get_content_width(&self) -> usize {
        let line_num_width = 3; // Minimum line number width
        (self.terminal_width as usize).saturating_sub(line_num_width + 1)
    }

    /// Set word wrap enabled/disabled and rebuild display caches
    pub fn set_wrap_enabled(&mut self, enabled: bool) -> Result<(), anyhow::Error> {
        if self.wrap_enabled != enabled {
            self.wrap_enabled = enabled;
            self.rebuild_display_caches()?;
            self.emit_view_event(crate::repl::events::ViewEvent::FullRedrawRequired);
        }
        Ok(())
    }

    /// Rebuild display caches for both panes
    fn rebuild_display_caches(&mut self) -> Result<(), anyhow::Error> {
        let content_width = self.get_content_width();

        // Rebuild request cache
        let request_lines = self.request_buffer.content().lines().to_vec();
        self.request_display_cache = crate::repl::models::build_display_cache(
            &request_lines,
            content_width,
            self.wrap_enabled,
        )
        .unwrap_or_else(|_| crate::repl::models::DisplayCache::new());

        // Rebuild response cache if there's response content
        let response_content = self.response.body();
        if !response_content.is_empty() {
            let response_lines: Vec<String> =
                response_content.lines().map(|s| s.to_string()).collect();
            self.response_display_cache = crate::repl::models::build_display_cache(
                &response_lines,
                content_width,
                self.wrap_enabled,
            )
            .unwrap_or_else(|_| crate::repl::models::DisplayCache::new());
        }

        // Synchronize display cursors after rebuilding caches
        // This is critical when wrap mode changes as cursor positions may be invalid
        self.sync_display_cursors();

        // Reset scroll offsets to ensure they remain valid after cache rebuild
        // When wrap mode changes, the number of display lines can change dramatically
        self.request_scroll_offset = (0, 0);
        self.response_scroll_offset = (0, 0);

        Ok(())
    }
}
