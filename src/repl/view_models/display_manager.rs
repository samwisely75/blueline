//! # Display Management
//!
//! Handles display cache management, word wrapping, and display coordinate calculations.
//! This module coordinates between logical content and display representation.

use crate::repl::events::{Pane, ViewEvent};
use crate::repl::geometry::Position;
use crate::repl::models::DisplayCache;
use crate::repl::view_models::core::{DisplayLineData, ViewModel};

impl ViewModel {
    /// Get display cache for a specific pane
    pub(super) fn get_display_cache(&self, pane: Pane) -> &DisplayCache {
        self.pane_manager.get_display_cache(pane)
    }

    /// Get display lines for rendering a specific pane
    /// Get display lines prepared for terminal rendering with visual selection support
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Get the display cache and scroll offsets for the specified pane
    /// 2. For each requested row in the viewport:
    ///    a. Calculate the actual display line index (accounting for vertical scroll)
    ///    b. If line exists, apply horizontal scrolling to get visible portion
    ///    c. Track how many characters were skipped for accurate logical column calculation
    ///    d. Package the visible content with metadata for visual selection rendering
    /// 3. Return display data including logical positions for selection highlighting
    ///
    /// CRITICAL FOR VISUAL SELECTION:
    /// - Must track the exact logical column for each visible character
    /// - Horizontal scrolling skips characters, not just display columns
    /// - Multi-byte characters complicate the mapping between display and logical positions
    pub fn get_display_lines_for_rendering(
        &self,
        pane: Pane,
        start_row: usize,
        row_count: usize,
    ) -> Vec<Option<DisplayLineData>> {
        // STEP 1: Gather display context - cache, scroll offsets, and dimensions
        let display_cache = self.get_display_cache(pane);
        let scroll_offset = if pane == self.pane_manager.current_pane_type() {
            self.pane_manager.get_current_scroll_offset()
        } else {
            // For non-current pane, access scroll offset directly (this should be made semantic later)
            Position::origin() // Simplified for now
        };
        let vertical_scroll_offset = scroll_offset.row;
        let horizontal_scroll_offset = scroll_offset.col;
        let content_width = self.get_content_width();
        let mut result = Vec::new();

        // STEP 2: Process each row in the viewport
        for row in 0..row_count {
            let display_line_idx = vertical_scroll_offset + start_row + row;

            if let Some(display_line) = display_cache.get_display_line(display_line_idx) {
                // STEP 2a: Get the full content of this display line
                let content = display_line.content();
                // STEP 2b: Apply horizontal scrolling to extract visible portion
                // CRITICAL: Track both the visible content AND how many characters were skipped
                // This is essential for calculating correct logical columns for visual selection
                let (visible_content, chars_skipped) =
                    if horizontal_scroll_offset > 0 || content.len() > content_width {
                        // HORIZONTAL SCROLLING ALGORITHM:
                        // 1. Iterate through characters tracking display column position
                        // 2. Skip characters until we reach horizontal_scroll_offset display columns
                        // 3. Collect characters that fit within content_width
                        // 4. Track the count of skipped characters (not display columns!)
                        use unicode_width::UnicodeWidthChar;
                        let mut result = String::new();
                        let mut current_col = 0;
                        let mut collecting = false;
                        let mut collected_width = 0;
                        let mut chars_skipped_count = 0;

                        for ch in content.chars() {
                            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);

                            // Start collecting when we reach the horizontal scroll offset
                            if !collecting && current_col >= horizontal_scroll_offset {
                                collecting = true;
                            }

                            // IMPORTANT: Count actual characters skipped, not display columns
                            // This is needed for correct logical column calculation
                            if !collecting {
                                chars_skipped_count += 1;
                            }

                            // Collect characters until we've filled the content width
                            if collecting {
                                if collected_width + char_width <= content_width {
                                    result.push(ch);
                                    collected_width += char_width;
                                } else {
                                    break; // Stop when adding this char would exceed content width
                                }
                            }

                            current_col += char_width;
                        }
                        (result, chars_skipped_count)
                    } else {
                        // No scrolling needed - use full content
                        (content.to_string(), 0)
                    };

                // STEP 2c: Prepare line number display
                // Show logical line number only for first segment of wrapped lines
                let line_number = if display_line.is_continuation {
                    None
                } else {
                    Some(display_line.logical_line + 1) // 1-based line numbers for display
                };

                // STEP 2d: Package display data with logical position information
                // BUGFIX #95: The logical start column must account for characters skipped
                // due to horizontal scrolling, not just the display columns scrolled
                //
                // DisplayLineData tuple structure:
                // - visible_content: The text to display (after horizontal scrolling)
                // - line_number: Optional line number (None for continuation lines)
                // - is_continuation: Whether this is a wrapped continuation line
                // - logical_start_col: The logical column where visible content starts
                // - logical_line: The logical line number (always provided)
                result.push(Some((
                    visible_content,
                    line_number,
                    display_line.is_continuation,
                    display_line.logical_start_col + chars_skipped, // Use character count, not display columns!
                    display_line.logical_line, // Always provide logical line number
                )));
            } else {
                // STEP 3: Handle empty rows (beyond content)
                // These will be rendered as tildes in the terminal
                result.push(None);
            }
        }

        // Return the prepared display data for rendering
        // Each element contains either display data or None (for tilde rows)
        result
    }

    /// Get cursor position for rendering
    pub fn get_cursor_for_rendering(&self, pane: Pane) -> (usize, usize) {
        let display_pos = if pane == self.pane_manager.current_pane_type() {
            self.pane_manager.get_current_display_cursor()
        } else {
            // For non-current pane, simplified for now
            Position::origin()
        };
        let scroll_offset = if pane == self.pane_manager.current_pane_type() {
            self.pane_manager.get_current_scroll_offset()
        } else {
            // For non-current pane, simplified for now
            Position::origin()
        };
        let vertical_offset = scroll_offset.row;
        let horizontal_offset = scroll_offset.col;

        // Calculate screen-relative position
        let screen_row = display_pos.row.saturating_sub(vertical_offset);
        let screen_col = display_pos.col.saturating_sub(horizontal_offset);

        (screen_row, screen_col)
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_logical_column_calculation_with_horizontal_scroll() {
        // Test that logical column calculation correctly accounts for
        // character count (not display columns) when horizontally scrolled

        // Create a test scenario with known content
        let content = "Hello あいうえお World"; // Mixed ASCII and multi-byte

        // Simulate horizontal scrolling by 2 display columns
        // "Hello " = 6 display columns, so we skip "He" (2 chars, 2 cols)
        let horizontal_offset = 2;
        let content_width = 20;

        // Apply the same logic as get_display_lines_for_rendering
        use unicode_width::UnicodeWidthChar;
        let mut result = String::new();
        let mut current_col = 0;
        let mut collecting = false;
        let mut collected_width = 0;
        let mut chars_skipped_count = 0;

        for ch in content.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);

            if !collecting && current_col >= horizontal_offset {
                collecting = true;
            }

            if !collecting {
                chars_skipped_count += 1;
            }

            if collecting {
                if collected_width + char_width <= content_width {
                    result.push(ch);
                    collected_width += char_width;
                } else {
                    break;
                }
            }

            current_col += char_width;
        }

        // Verify we skipped 2 characters (not 2 display columns)
        assert_eq!(
            chars_skipped_count, 2,
            "Should skip 2 characters for 2 display columns of ASCII"
        );
        assert_eq!(
            &result[0..3],
            "llo",
            "Visible content should start with 'llo'"
        );
    }

    #[test]
    fn test_logical_column_with_multibyte_horizontal_scroll() {
        // Test with multi-byte characters that occupy 2 display columns each
        let content = "あいうえお"; // 5 characters, 10 display columns

        // Scroll by 4 display columns (should skip 2 double-width chars)
        let horizontal_offset = 4;
        let content_width = 10;

        use unicode_width::UnicodeWidthChar;
        let mut result = String::new();
        let mut current_col = 0;
        let mut collecting = false;
        let mut collected_width = 0;
        let mut chars_skipped_count = 0;

        for ch in content.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);

            if !collecting && current_col >= horizontal_offset {
                collecting = true;
            }

            if !collecting {
                chars_skipped_count += 1;
            }

            if collecting {
                if collected_width + char_width <= content_width {
                    result.push(ch);
                    collected_width += char_width;
                } else {
                    break;
                }
            }

            current_col += char_width;
        }

        // Should skip 2 characters (あい) which occupy 4 display columns
        assert_eq!(
            chars_skipped_count, 2,
            "Should skip 2 double-width characters"
        );
        assert_eq!(
            result, "うえお",
            "Visible content should be last 3 characters"
        );
    }
}
