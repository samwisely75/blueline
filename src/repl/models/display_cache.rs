//! # Display Cache for Word Wrap Support
//!
//! Provides display line caching for efficient word wrap rendering and cursor positioning.
//! Maps logical lines to display lines with position tracking for navigation.

use std::collections::HashMap;
use std::time::Instant;

/// Type alias for display position tuples (display_line, display_column)
pub type DisplayPosition = (usize, usize);

/// Type alias for logical-to-display line mapping
pub type LogicalToDisplayMap = HashMap<usize, Vec<usize>>;

/// Type alias for logical position tuple (logical_line, logical_column)
pub type LogicalPosition = (usize, usize);

/// Pre-calculated display line with positioning metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayLine {
    /// The text content to display on this line
    pub content: String,
    /// Index of the logical line this display line represents
    pub logical_line: usize,
    /// Starting column position in the logical line
    pub logical_start_col: usize,
    /// Ending column position in the logical line (exclusive)
    pub logical_end_col: usize,
    /// True if this is a continuation of a wrapped line
    pub is_continuation: bool,
}

/// Cache for pre-calculated display lines with fast lookup
#[derive(Debug, Clone)]
pub struct DisplayCache {
    /// All display lines in order
    pub display_lines: Vec<DisplayLine>,
    /// Map logical line index to display line indices
    pub logical_to_display: LogicalToDisplayMap,
    /// Total number of display lines
    pub total_display_lines: usize,
    /// Content width used for this cache
    pub content_width: usize,
    /// Content hash for invalidation detection
    pub content_hash: u64,
    /// When this cache was generated
    pub generated_at: Instant,
    /// Whether this cache is valid
    pub is_valid: bool,
    /// Whether word wrap is enabled for this cache
    pub wrap_enabled: bool,
}

impl DisplayCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            display_lines: Vec::new(),
            logical_to_display: HashMap::new(),
            total_display_lines: 0,
            content_width: 0,
            content_hash: 0,
            generated_at: Instant::now(),
            is_valid: false,
            wrap_enabled: false,
        }
    }

    /// Check if cache is valid for given content, width, and wrap mode
    pub fn is_valid_for(&self, content_hash: u64, width: usize, wrap_enabled: bool) -> bool {
        self.is_valid
            && self.content_hash == content_hash
            && self.content_width == width
            && self.wrap_enabled == wrap_enabled
    }

    /// Invalidate the cache
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Convert logical cursor position to display position using cache
    pub fn logical_to_display_position(
        &self,
        logical_line: usize,
        logical_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid {
            return None;
        }

        // Find display lines for this logical line
        let display_indices = self.logical_to_display.get(&logical_line)?;

        for &display_idx in display_indices {
            if let Some(display_line) = self.display_lines.get(display_idx) {
                tracing::debug!(
                    "logical_to_display_position: checking display_idx={}, logical_col={}, start_col={}, end_col={}",
                    display_idx, logical_col, display_line.logical_start_col, display_line.logical_end_col
                );

                // Check if cursor falls within this display line segment
                if logical_col >= display_line.logical_start_col
                    && logical_col < display_line.logical_end_col
                {
                    let display_col = logical_col - display_line.logical_start_col;
                    tracing::debug!(
                        "logical_to_display_position: found match in range, returning ({}, {})",
                        display_idx,
                        display_col
                    );
                    return Some((display_idx, display_col));
                }
                // BUGFIX: Handle end-of-line case - when logical_col equals logical_end_col,
                // it should map to the beginning of the NEXT display line (column 0),
                // not the end of the current display line. This fixes the boundary condition
                // where cursor gets stuck at column 1 on wrapped continuation lines.
                if logical_col == display_line.logical_end_col
                    && display_line.logical_end_col > display_line.logical_start_col
                {
                    // Check if there's a next display line for this logical line
                    let next_display_idx = display_idx + 1;
                    if next_display_idx < self.display_lines.len() {
                        if let Some(next_display_line) = self.display_lines.get(next_display_idx) {
                            // If the next display line is a continuation of the same logical line,
                            // position cursor at the beginning of it (column 0)
                            if next_display_line.logical_line == display_line.logical_line
                                && next_display_line.is_continuation
                            {
                                tracing::debug!(
                                    "logical_to_display_position: found end-of-line match, moving to next continuation line ({}, 0)",
                                    next_display_idx
                                );
                                return Some((next_display_idx, 0));
                            }
                        }
                    }

                    // If no next continuation line, position at end of current line (fallback)
                    let display_col = display_line.logical_end_col - display_line.logical_start_col;
                    tracing::debug!(
                        "logical_to_display_position: found end of logical line, returning ({}, {})",
                        display_idx, display_col
                    );
                    return Some((display_idx, display_col));
                }
            }
        }

        // Fallback to last display line of this logical line
        if let Some(&last_display_idx) = display_indices.last() {
            if let Some(display_line) = self.display_lines.get(last_display_idx) {
                let display_col = (display_line.content.chars().count())
                    .min(logical_col.saturating_sub(display_line.logical_start_col));
                return Some((last_display_idx, display_col));
            }
        }

        None
    }

    /// Convert display position to logical position using cache
    pub fn display_to_logical_position(
        &self,
        display_line: usize,
        display_col: usize,
    ) -> Option<LogicalPosition> {
        if !self.is_valid {
            tracing::debug!("display_to_logical_position: cache invalid");
            return None;
        }

        let display_info = self.display_lines.get(display_line)?;
        let logical_line = display_info.logical_line;

        // BUGFIX: Clamp display column to valid cursor positions to prevent horizontal scrolling issues
        // For a line with N characters, valid cursor positions are 0 to N-1 (on each character).
        // Position N would be after the last character and can trigger unwanted horizontal scrolling.
        // When moving across wrapped line segments, display_col might exceed valid positions.
        let content_length = display_info.content.chars().count();
        let clamped_display_col = display_col.min(content_length);
        let logical_col = display_info.logical_start_col + clamped_display_col;

        tracing::debug!(
            "display_to_logical_position: display=({}, {}) -> logical=({}, {}) [content_len={}, start_col={}, clamped_col={}]",
            display_line, display_col, logical_line, logical_col,
            content_length, display_info.logical_start_col, clamped_display_col
        );

        Some((logical_line, logical_col))
    }

    /// Get display line content and metadata
    pub fn get_display_line(&self, display_idx: usize) -> Option<&DisplayLine> {
        if !self.is_valid {
            return None;
        }
        self.display_lines.get(display_idx)
    }

    /// Move cursor up by one display line
    pub fn move_up(
        &self,
        current_display_line: usize,
        desired_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid || current_display_line == 0 {
            return None;
        }

        let target_display_line = current_display_line - 1;
        let target_display_info = self.display_lines.get(target_display_line)?;

        // Try to maintain column position, but clamp to line length
        let target_col = desired_col.min(target_display_info.content.chars().count());

        Some((target_display_line, target_col))
    }

    /// Move cursor down by one display line
    pub fn move_down(
        &self,
        current_display_line: usize,
        desired_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid || current_display_line >= self.display_lines.len().saturating_sub(1) {
            return None;
        }

        let target_display_line = current_display_line + 1;
        let target_display_info = self.display_lines.get(target_display_line)?;

        // Try to maintain column position, but clamp to line length
        let target_col = desired_col.min(target_display_info.content.chars().count());

        Some((target_display_line, target_col))
    }

    /// Get total number of display lines
    pub fn display_line_count(&self) -> usize {
        self.total_display_lines
    }

    /// Check if a logical line has multiple display lines (is wrapped)
    pub fn is_logical_line_wrapped(&self, logical_line: usize) -> bool {
        if let Some(display_indices) = self.logical_to_display.get(&logical_line) {
            display_indices.len() > 1
        } else {
            false
        }
    }
}

impl Default for DisplayCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a wrapped line segment
#[derive(Debug, Clone)]
struct WrappedSegment {
    content: String,
    logical_start: usize,
    logical_end: usize,
}

/// Build a display cache from logical lines
pub fn build_display_cache(
    lines: &[String],
    content_width: usize,
    wrap_enabled: bool,
) -> anyhow::Result<DisplayCache> {
    let content_hash = calculate_content_hash(lines);
    let mut display_lines = Vec::new();
    let mut logical_to_display = HashMap::new();

    for (logical_idx, line) in lines.iter().enumerate() {
        let wrapped_segments = if wrap_enabled {
            wrap_line_with_positions(line, content_width)
        } else {
            // No wrap: 1:1 mapping
            vec![WrappedSegment {
                content: line.clone(),
                logical_start: 0,
                logical_end: line.chars().count(),
            }]
        };

        let mut display_indices = Vec::new();

        for (segment_idx, segment_info) in wrapped_segments.iter().enumerate() {
            let display_idx = display_lines.len();
            display_indices.push(display_idx);

            let display_line = DisplayLine {
                content: segment_info.content.clone(),
                logical_line: logical_idx,
                logical_start_col: segment_info.logical_start,
                logical_end_col: segment_info.logical_end,
                is_continuation: segment_idx > 0,
            };

            display_lines.push(display_line);
        }

        logical_to_display.insert(logical_idx, display_indices);
    }

    Ok(DisplayCache {
        total_display_lines: display_lines.len(),
        display_lines,
        logical_to_display,
        content_width,
        content_hash,
        generated_at: Instant::now(),
        is_valid: true,
        wrap_enabled,
    })
}

/// Wrap a line into segments with accurate position tracking
fn wrap_line_with_positions(line: &str, content_width: usize) -> Vec<WrappedSegment> {
    if content_width == 0 {
        return vec![WrappedSegment {
            content: line.to_string(),
            logical_start: 0,
            logical_end: line.chars().count(),
        }];
    }

    let mut segments = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = line.chars().collect();

    while current_pos < chars.len() {
        let remaining_chars = chars.len() - current_pos;

        if remaining_chars <= content_width {
            // Remaining text fits in one line
            let segment = WrappedSegment {
                content: chars[current_pos..].iter().collect(),
                logical_start: current_pos,
                logical_end: chars.len(),
            };
            segments.push(segment);
            break;
        }

        // Calculate the end position for this segment
        let segment_end = (current_pos + content_width).min(chars.len());
        let mut break_point = content_width;

        // Look backwards from the segment end for a space or word boundary
        for i in (current_pos..segment_end).rev() {
            if chars[i] == ' ' || chars[i] == '\t' {
                break_point = i - current_pos;
                break;
            }
        }

        // If no word boundary found and we would break a word, force break at content_width
        if break_point == content_width && current_pos + content_width < chars.len() {
            // This is correct - break at content_width
        }

        let actual_end = current_pos + break_point;
        let segment_content: String = chars[current_pos..actual_end].iter().collect();

        let segment = WrappedSegment {
            content: segment_content,
            logical_start: current_pos,
            logical_end: actual_end,
        };
        segments.push(segment);

        current_pos = actual_end;

        // Skip whitespace at the beginning of next line, but track this in positions
        while current_pos < chars.len() && (chars[current_pos] == ' ' || chars[current_pos] == '\t')
        {
            current_pos += 1;
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

/// Calculate a simple hash of the content for invalidation detection
fn calculate_content_hash(lines: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    lines.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_cache_should_create_empty() {
        let cache = DisplayCache::new();
        assert!(!cache.is_valid);
        assert_eq!(cache.display_lines.len(), 0);
    }

    #[test]
    fn build_display_cache_should_work_without_wrap() {
        let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        assert!(cache.is_valid);
        assert!(!cache.wrap_enabled);
        assert_eq!(cache.display_lines.len(), 2);
        assert_eq!(cache.display_lines[0].content, "Line 1");
        assert_eq!(cache.display_lines[1].content, "Line 2");
        assert!(!cache.display_lines[0].is_continuation);
        assert!(!cache.display_lines[1].is_continuation);
    }

    #[test]
    fn build_display_cache_should_work_with_wrap() {
        let lines = vec!["This is a very long line that should wrap".to_string()];
        let cache = build_display_cache(&lines, 20, true).unwrap();

        assert!(cache.is_valid);
        assert!(cache.wrap_enabled);
        assert!(cache.display_lines.len() > 1); // Should be wrapped

        // First segment should not be continuation
        assert!(!cache.display_lines[0].is_continuation);

        // Following segments should be continuations
        for i in 1..cache.display_lines.len() {
            assert!(cache.display_lines[i].is_continuation);
        }
    }

    #[test]
    fn logical_to_display_position_should_work() {
        let lines = vec!["Hello world this is a test".to_string()];
        let cache = build_display_cache(&lines, 10, true).unwrap();

        // Position at start should map to display (0, 0)
        let (display_line, display_col) = cache.logical_to_display_position(0, 0).unwrap();
        assert_eq!(display_line, 0);
        assert_eq!(display_col, 0);

        // Position in middle should map to appropriate display line
        if let Some((display_line, display_col)) = cache.logical_to_display_position(0, 15) {
            assert!(display_line > 0); // Should be on a wrapped line
            assert!(display_col < 10); // Within the content width
        }
    }

    #[test]
    fn display_to_logical_position_should_work() {
        let lines = vec!["Hello world test".to_string()];
        let cache = build_display_cache(&lines, 10, true).unwrap();

        // First display line should map back to logical line 0
        let (logical_line, logical_col) = cache.display_to_logical_position(0, 5).unwrap();
        assert_eq!(logical_line, 0);
        assert_eq!(logical_col, 5);
    }

    #[test]
    fn move_up_down_should_work() {
        let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        // Move down from first line
        let (new_line, new_col) = cache.move_down(0, 3).unwrap();
        assert_eq!(new_line, 1);
        assert_eq!(new_col, 3);

        // Move up from second line
        let (new_line, new_col) = cache.move_up(1, 3).unwrap();
        assert_eq!(new_line, 0);
        assert_eq!(new_col, 3);
    }

    #[test]
    fn cache_invalidation_should_work() {
        let lines = vec!["Test".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        assert!(cache.is_valid_for(calculate_content_hash(&lines), 80, false));
        assert!(!cache.is_valid_for(calculate_content_hash(&lines), 60, false)); // Different width
        assert!(!cache.is_valid_for(calculate_content_hash(&lines), 80, true)); // Different wrap mode
    }
}
