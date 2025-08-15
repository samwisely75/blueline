//! Display cache management and line wrapping logic for PaneState
//!
//! This module contains methods for:
//! - Building and maintaining the display cache
//! - Line wrapping calculations with word boundaries
//! - Display width calculations for multi-byte characters
//! - Line number width management

use crate::repl::geometry::Dimensions;
use crate::repl::models::{DisplayCache, DisplayLine};
use std::collections::HashMap;
use std::time::Instant;

use super::{PaneState, WrappedSegment, MIN_LINE_NUMBER_WIDTH};

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
        use crate::repl::models::display_cache::*;
        use crate::repl::models::display_char::DisplayChar;

        // Ensure word boundaries are calculated for all lines
        let character_buffer = self.buffer.content_mut().character_buffer_mut();
        let line_count = character_buffer.line_count();

        tracing::debug!("Pre-calculating word boundaries for {} lines", line_count);

        // Pre-calculate word boundaries for all lines
        for line_idx in 0..line_count {
            character_buffer.get_line_word_boundaries(line_idx);
            if let Some(line) = character_buffer.get_line(line_idx) {
                let word_starts = line
                    .chars()
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_word_start)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                let word_ends = line
                    .chars()
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_word_end)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();

                if !word_starts.is_empty() || !word_ends.is_empty() {
                    tracing::debug!(
                        "Line {}: '{}' -> word_starts={:?}, word_ends={:?}",
                        line_idx,
                        line.to_string().chars().take(50).collect::<String>(), // First 50 chars
                        word_starts,
                        word_ends
                    );
                }
            }
        }
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
        use crate::repl::models::buffer_char::BufferLine;
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
}