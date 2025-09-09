//! Display cache management and line wrapping logic for PaneState
//!
//! This module contains methods for:
//! - Building and maintaining the display cache
//! - Line wrapping calculations with word boundaries
//! - Display width calculations for multi-byte characters
//! - Line number width management

use crate::repl::models::coordinates::geometry::Dimensions;
use crate::repl::models::{DisplayCache, DisplayLine};
use std::collections::HashMap;
use std::time::Instant;

use super::{PaneState, WrappedSegment, MIN_LINE_NUMBER_WIDTH};
use crate::repl::models::buffer::buffer_char::SegmentationState;
use crate::repl::services::AsyncWordSegmenter;

impl PaneState {
    /// Build the display cache for text rendering with proper word boundaries
    pub fn build_display_cache(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) {
        tracing::info!(
            "Building display cache with unicode-segmentation word boundaries: content_width={}, wrap_enabled={}, tab_width={}",
            content_width,
            wrap_enabled,
            tab_width
        );

        // Use CharacterBuffer directly to preserve word boundary information
        self.display_cache = self
            .build_display_cache_from_character_buffer(content_width, wrap_enabled, tab_width)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to build display cache with word boundaries: {}", e);
                DisplayCache::new()
            });

        tracing::debug!(
            "Display cache built: {} display lines, {} logical lines mapped",
            self.display_cache.display_lines.len(),
            self.display_cache.logical_to_display.len()
        );
    }

    /// Build display cache from CharacterBuffer preserving word boundaries
    fn build_display_cache_from_character_buffer(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> anyhow::Result<DisplayCache> {
        use crate::repl::models::display::display_cache::*;
        use crate::repl::models::display::display_char::DisplayChar;

        // Ensure word boundaries are calculated for all lines
        let character_buffer = self.buffer.content_mut().character_buffer_mut();
        let line_count = character_buffer.line_count();

        tracing::debug!(
            "Building display cache for {} lines (word boundaries calculated lazily)",
            line_count
        );

        // Skip aggressive pre-calculation of word boundaries for all lines
        // Word boundaries will be calculated lazily when lines are actually accessed
        // This prevents performance issues with large responses containing very long lines
        let mut display_lines = Vec::new();
        let mut logical_to_display = HashMap::new();

        for logical_idx in 0..line_count {
            if let Some(buffer_line) = character_buffer.get_line(logical_idx) {
                let line_text = buffer_line.to_string();

                let wrapped_segments = if wrap_enabled {
                    Self::wrap_line_with_positions(&line_text, content_width)
                } else {
                    vec![WrappedSegment {
                        content: line_text.clone(),
                        logical_start: 0,
                        logical_end: buffer_line.char_count(),
                    }]
                };

                let mut display_indices = Vec::new();

                for (segment_idx, segment_info) in wrapped_segments.iter().enumerate() {
                    let display_idx = display_lines.len();
                    display_indices.push(display_idx);

                    // Create DisplayChars from the segment, preserving word boundaries
                    let mut display_chars = Vec::new();
                    let mut current_screen_col = 0;

                    // Extract the relevant BufferChars from the line
                    for logical_col in segment_info.logical_start..segment_info.logical_end {
                        if let Some(buffer_char) = buffer_line.get_char(logical_col) {
                            let display_char = DisplayChar::from_buffer_char_with_tab_width(
                                buffer_char.clone(),
                                (display_idx, current_screen_col),
                                tab_width,
                            );
                            current_screen_col += display_char.display_width();
                            display_chars.push(display_char);
                        }
                    }

                    let display_line = DisplayLine::new(
                        display_chars,
                        logical_idx,
                        segment_info.logical_start,
                        segment_info.logical_end,
                        segment_idx > 0,
                    );

                    display_lines.push(display_line);
                }

                logical_to_display.insert(logical_idx, display_indices);
            }
        }

        Ok(DisplayCache {
            total_display_lines: display_lines.len(),
            display_lines,
            logical_to_display,
            content_width,
            content_hash: 0, // Not used with eager invalidation strategy
            generated_at: Instant::now(),
            is_valid: true,
            wrap_enabled,
        })
    }

    /// Calculate display width for a range of buffer characters
    fn calculate_display_width(
        buffer_chars: &[crate::repl::models::buffer_char::BufferChar],
    ) -> usize {
        use unicode_width::UnicodeWidthChar;
        buffer_chars
            .iter()
            .map(|bc| UnicodeWidthChar::width(bc.ch).unwrap_or(0))
            .sum()
    }

    /// Wrap a line into segments with position tracking
    fn wrap_line_with_positions(line: &str, content_width: usize) -> Vec<WrappedSegment> {
        if content_width == 0 {
            return vec![WrappedSegment {
                content: line.to_string(),
                logical_start: 0,
                logical_end: line.chars().count(),
            }];
        }

        // Convert line to BufferChars for accurate display width calculation
        use crate::repl::models::buffer::buffer_char::BufferLine;
        let buffer_line = BufferLine::from_string(line);
        let buffer_chars = buffer_line.chars();

        let mut segments = Vec::new();
        let mut current_char_pos = 0;
        let total_chars = buffer_chars.len();

        while current_char_pos < total_chars {
            let mut current_display_width = 0;
            let mut segment_end_char_pos = current_char_pos;
            let mut last_word_boundary_char_pos = None;

            // Find how many characters fit within content_width display columns
            while segment_end_char_pos < total_chars && current_display_width < content_width {
                let buffer_char = &buffer_chars[segment_end_char_pos];
                use unicode_width::UnicodeWidthChar;
                let char_display_width = UnicodeWidthChar::width(buffer_char.ch).unwrap_or(0);

                // Check if adding this character would exceed the content width
                if current_display_width + char_display_width > content_width {
                    break;
                }

                // Mark word boundaries for better wrapping
                if buffer_char.ch.is_whitespace() {
                    last_word_boundary_char_pos = Some(segment_end_char_pos + 1);
                }

                current_display_width += char_display_width;
                segment_end_char_pos += 1;
            }

            // If we haven't advanced and we're not at the last character, force advance by one
            // to prevent infinite loops with zero-width characters
            if segment_end_char_pos == current_char_pos && current_char_pos < total_chars {
                segment_end_char_pos = current_char_pos + 1;
            }

            // Try to break at word boundary if possible (only if we have more characters to process)
            let actual_end = if segment_end_char_pos < total_chars {
                if let Some(word_boundary) = last_word_boundary_char_pos {
                    if word_boundary > current_char_pos {
                        word_boundary
                    } else {
                        segment_end_char_pos
                    }
                } else {
                    segment_end_char_pos
                }
            } else {
                segment_end_char_pos
            };

            // Extract the segment content
            let segment_content: String = buffer_chars[current_char_pos..actual_end]
                .iter()
                .map(|bc| bc.ch)
                .collect();

            segments.push(WrappedSegment {
                content: segment_content,
                logical_start: current_char_pos,
                logical_end: actual_end,
            });

            current_char_pos = actual_end;
        }

        // WRAP MODE CURSOR POSITIONING FIX: Create an empty continuation segment for cursor positioning
        // When content exactly fills display lines, we need a place for the cursor to wrap to
        if !segments.is_empty() {
            let last_segment = segments.last().unwrap();
            // Only process if we've reached the end of all characters
            if last_segment.logical_end == total_chars && total_chars > 0 {
                // Check if the last segment exactly fills a display line
                let segment_display_width = Self::calculate_display_width(
                    &buffer_chars[last_segment.logical_start..last_segment.logical_end],
                );

                // If this segment exactly fills the content width, create empty continuation
                if segment_display_width == content_width {
                    segments.push(WrappedSegment {
                        content: String::new(),
                        logical_start: total_chars,
                        logical_end: total_chars,
                    });
                }
            }
        }

        if segments.is_empty() {
            segments.push(WrappedSegment {
                content: String::new(),
                logical_start: 0,
                logical_end: 0,
            });
        }

        segments
    }

    // ========================================
    // Dimension Management
    // ========================================

    /// Get content width for this pane
    pub fn get_content_width(&self) -> usize {
        self.pane_dimensions.width
    }

    /// Update pane dimensions (for terminal resize)
    pub fn update_dimensions(&mut self, width: usize, height: usize) {
        self.pane_dimensions = Dimensions::new(width, height);
    }

    // ========================================
    // Line Number Management
    // ========================================

    /// Update line number width based on current content
    pub fn update_line_number_width(&mut self) {
        let content = self.buffer.content().get_text();
        let line_count = if content.is_empty() {
            1 // At least show line 1 even for empty content
        } else {
            content.lines().count().max(1)
        };

        // Calculate width needed for the largest line number to prevent cursor positioning bugs
        let width = line_count.to_string().len();

        // Minimum width as specified in the requirements (never smaller than 3)
        self.line_number_width = width.max(MIN_LINE_NUMBER_WIDTH);
    }

    /// Get current line number width for this pane
    pub fn get_line_number_width(&self) -> usize {
        self.line_number_width
    }

    // ========================================
    // Async Word Segmentation Support
    // ========================================

    /// Request async segmentation for lines in the viewport
    /// This should be called after build_display_cache to trigger segmentation for visible lines
    pub fn request_viewport_segmentation(
        &mut self,
        async_segmenter: &AsyncWordSegmenter,
        viewport_height: usize,
    ) {
        // Calculate which logical lines are potentially visible based on display cache
        let visible_logical_lines = self.get_visible_logical_lines(viewport_height);

        // Process each line
        let character_buffer = self.buffer.content_mut().character_buffer_mut();
        for logical_line_idx in visible_logical_lines {
            if let Some(line) = character_buffer.get_line_mut(logical_line_idx) {
                // Only request segmentation if not already done or in progress
                match line.segmentation_state() {
                    SegmentationState::NotRequested => {
                        let text = line.to_string();
                        let generation_id = line.generation_id();

                        // Mark as pending to avoid duplicate requests
                        line.mark_segmentation_pending();

                        // Request async segmentation (non-blocking)
                        if let Err(e) = async_segmenter.request_segmentation(
                            logical_line_idx,
                            text,
                            generation_id,
                        ) {
                            tracing::warn!(
                                "Failed to request segmentation for line {}: {}",
                                logical_line_idx,
                                e
                            );
                            // Reset state on failure
                            line.invalidate_word_boundaries();
                        } else {
                            tracing::debug!(
                                "Requested async segmentation for line {} (gen {})",
                                logical_line_idx,
                                generation_id
                            );
                        }
                    }
                    SegmentationState::Pending | SegmentationState::Complete => {
                        // Already handled, skip
                    }
                }
            }
        }
    }

    /// Process completed async segmentation results
    /// Returns true if any results were applied (indicating a redraw may be needed)
    pub fn process_segmentation_results(&mut self, async_segmenter: &AsyncWordSegmenter) -> bool {
        let results = async_segmenter.try_recv_results();
        if results.is_empty() {
            return false;
        }

        let mut any_applied = false;
        let character_buffer = self.buffer.content_mut().character_buffer_mut();

        for result in results {
            if let Some(line) = character_buffer.get_line_mut(result.line_id) {
                // Apply segmentation flags (includes generation validation)
                line.apply_segmentation_flags(result.flags, result.generation_id);
                any_applied = true;
                tracing::debug!(
                    "Applied segmentation result for line {} (gen {})",
                    result.line_id,
                    result.generation_id
                );
            }
        }

        any_applied
    }

    /// Get the range of logical lines that are potentially visible in the viewport
    fn get_visible_logical_lines(&self, viewport_height: usize) -> Vec<usize> {
        let mut visible_lines = Vec::new();

        // Get the current scroll position
        let scroll_start_display_line = self.scroll_offset.row;
        let scroll_end_display_line = scroll_start_display_line + viewport_height;

        // Map display lines back to logical lines
        for display_line_idx in scroll_start_display_line
            ..scroll_end_display_line.min(self.display_cache.total_display_lines)
        {
            if let Some(display_line) = self.display_cache.display_lines.get(display_line_idx) {
                let logical_line_idx = display_line.logical_line;
                if !visible_lines.contains(&logical_line_idx) {
                    visible_lines.push(logical_line_idx);
                }
            }
        }

        // Also include a buffer around the viewport for smoother scrolling
        let buffer_lines = 10; // Lines above and below viewport to pre-segment
        let character_buffer = self.buffer.content().character_buffer();
        let total_logical_lines = character_buffer.line_count();

        if let (Some(&first_visible), Some(&last_visible)) =
            (visible_lines.first(), visible_lines.last())
        {
            // Add lines before viewport
            let start_with_buffer = first_visible.saturating_sub(buffer_lines);
            for line_idx in start_with_buffer..first_visible {
                visible_lines.push(line_idx);
            }

            // Add lines after viewport
            let end_with_buffer = (last_visible + buffer_lines + 1).min(total_logical_lines);
            for line_idx in (last_visible + 1)..end_with_buffer {
                visible_lines.push(line_idx);
            }
        }

        // Sort and deduplicate
        visible_lines.sort_unstable();
        visible_lines.dedup();

        tracing::debug!(
            "Viewport covers {} logical lines: {:?}",
            visible_lines.len(),
            if visible_lines.len() <= 10 {
                format!("{visible_lines:?}")
            } else {
                format!("{:?}...", &visible_lines[0..5])
            }
        );

        visible_lines
    }
}
