//! # Display Cache for Word Wrap Support
//!
//! Provides display line caching for efficient word wrap rendering and cursor positioning.
//! Maps logical lines to display lines with position tracking for navigation.

use crate::repl::models::buffer_char::BufferLine;
use crate::repl::models::display_line::DisplayLine;
use crate::repl::models::geometry::Position;
use std::collections::HashMap;
use std::time::Instant;

/// Type alias for display position (display_line, display_column)
pub type DisplayPosition = Position;

/// Type alias for logical-to-display line mapping
pub type LogicalToDisplayMap = HashMap<usize, Vec<usize>>;

/// Type alias for logical position (logical_line, logical_column)  
pub type LogicalPosition = Position;

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
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Find all display line segments that make up the target logical line
    /// 2. Determine which segment contains the target logical column
    /// 3. Convert the logical column to display column, accounting for multibyte character widths
    /// 4. Handle special cases like line wrapping boundaries and end-of-line positions
    pub fn logical_to_display_position(
        &self,
        logical_line: usize,
        logical_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid {
            return None;
        }

        // Find display lines for this logical line
        let display_line_indices = self.logical_to_display.get(&logical_line)?;

        // Find which display line contains the logical column
        for &display_idx in display_line_indices {
            let display_line = self.display_lines.get(display_idx)?;

            // Check if this display line contains the logical column
            if logical_col >= display_line.logical_start_col
                && logical_col <= display_line.logical_end_col
            {
                // Map the logical column to display column using character display widths
                // Adjust logical_col to be relative to this display line's start
                let relative_col = logical_col - display_line.logical_start_col;
                let display_col = display_line.logical_index_to_display_col(relative_col);
                return Some(Position::new(display_idx, display_col));
            }
        }

        // If we couldn't find an exact match, position at end of last segment
        if let Some(&last_display_idx) = display_line_indices.last() {
            if let Some(last_display_line) = self.display_lines.get(last_display_idx) {
                let display_width = last_display_line.display_width();
                return Some(Position::new(last_display_idx, display_width));
            }
        }

        None
    }

    /// Convert display position to logical position using cache
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Find the display line at the given display row
    /// 2. Map the display column to logical column using character display widths
    /// 3. Combine the logical line index with logical column
    /// 4. Handle special cases like end-of-line and continuation lines
    pub fn display_to_logical_position(
        &self,
        display_line: usize,
        display_col: usize,
    ) -> Option<LogicalPosition> {
        if !self.is_valid {
            return None;
        }

        let line_info = self.display_lines.get(display_line)?;
        let logical_index = line_info.display_col_to_logical_index(display_col);

        Some(Position::new(line_info.logical_line, logical_index))
    }

    /// Get a specific display line by index
    pub fn get_display_line(&self, index: usize) -> Option<&DisplayLine> {
        self.display_lines.get(index)
    }

    /// Get the count of display lines
    pub fn display_line_count(&self) -> usize {
        self.total_display_lines
    }

    /// Move cursor up one display line
    pub fn move_up(&self, display_row: usize, display_col: usize) -> Option<DisplayPosition> {
        if display_row == 0 {
            return None;
        }

        let new_row = display_row - 1;
        let line_info = self.get_display_line(new_row)?;

        // Clamp column to line width for proper cursor positioning
        let new_col = display_col.min(line_info.display_width());

        Some(Position::new(new_row, new_col))
    }

    /// Move cursor down one display line
    pub fn move_down(&self, display_row: usize, display_col: usize) -> Option<DisplayPosition> {
        let new_row = display_row + 1;
        if new_row >= self.display_line_count() {
            return None;
        }

        let line_info = self.get_display_line(new_row)?;

        // Clamp column to line width for proper cursor positioning
        let new_col = display_col.min(line_info.display_width());

        Some(Position::new(new_row, new_col))
    }
}

impl Default for DisplayCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate a simple hash of content for cache invalidation
pub fn calculate_content_hash(lines: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for line in lines {
        line.hash(&mut hasher);
    }
    hasher.finish()
}

/// Segment of a wrapped line with position tracking
struct WrappedSegment {
    content: String,
    logical_start: usize, // Starting character position in the logical line
    logical_end: usize,   // Ending character position (exclusive)
}

/// Build display cache from logical lines with optional word wrap
///
/// HIGH-LEVEL LOGIC:
/// 1. Calculate content hash for cache invalidation
/// 2. Process each logical line, potentially wrapping into multiple display lines
/// 3. Build mapping from logical to display line indices
/// 4. Create DisplayLine structures with proper positioning metadata
/// 5. Return complete cache with all display lines and mappings
///
/// WRAP ALGORITHM:
/// - If wrap disabled: one logical line = one display line
/// - If wrap enabled: break lines at word boundaries when exceeding content_width
/// - Track character positions for accurate cursor mapping
pub fn build_display_cache(
    lines: &[String],
    content_width: usize,
    wrap_enabled: bool,
) -> Result<DisplayCache, String> {
    if lines.is_empty() {
        return Ok(DisplayCache::new());
    }

    let content_hash = calculate_content_hash(lines);
    let mut display_lines = Vec::new();
    let mut logical_to_display = HashMap::new();

    for (logical_idx, line) in lines.iter().enumerate() {
        let mut display_indices = Vec::new();

        let segments = if wrap_enabled && content_width > 0 {
            wrap_line_with_positions(line, content_width)
        } else {
            vec![WrappedSegment {
                content: line.clone(),
                logical_start: 0,
                logical_end: line.chars().count(),
            }]
        };

        for (segment_idx, segment_info) in segments.iter().enumerate() {
            let display_idx = display_lines.len();
            display_indices.push(display_idx);

            // Create DisplayLine for this segment
            #[allow(deprecated)]
            let display_line = DisplayLine::from_content(
                &segment_info.content,
                logical_idx,
                segment_info.logical_start,
                segment_info.logical_end,
                segment_idx > 0,
            );

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

/// Wrap a line into segments with accurate position tracking using character display widths
fn wrap_line_with_positions(line: &str, content_width: usize) -> Vec<WrappedSegment> {
    if content_width == 0 {
        return vec![WrappedSegment {
            content: line.to_string(),
            logical_start: 0,
            logical_end: line.chars().count(),
        }];
    }

    // Convert line to BufferChars for accurate display width calculation
    let buffer_line = BufferLine::from_string(line);
    let buffer_chars = buffer_line.chars();

    let mut segments = Vec::new();
    let mut current_char_pos = 0;
    let total_chars = buffer_chars.len();

    while current_char_pos < total_chars {
        let mut current_display_width = 0;
        let mut segment_end_char_pos = current_char_pos;
        let mut last_word_boundary_char_pos = None;

        // Find the maximum number of characters that fit in content_width
        while segment_end_char_pos < total_chars {
            let char_display_width = buffer_chars[segment_end_char_pos].display_width;

            // Check if adding this character would exceed the width
            if current_display_width + char_display_width > content_width {
                // If we have a previous word boundary, break there
                if let Some(boundary_pos) = last_word_boundary_char_pos {
                    segment_end_char_pos = boundary_pos;
                }
                break;
            }

            current_display_width += char_display_width;

            // Track word boundaries for better breaking points
            if buffer_chars[segment_end_char_pos].is_word_start
                || buffer_chars[segment_end_char_pos].ch.is_whitespace()
            {
                last_word_boundary_char_pos = Some(segment_end_char_pos + 1);
            }

            segment_end_char_pos += 1;
        }

        // Extract the segment content
        let segment_text: String = buffer_chars[current_char_pos..segment_end_char_pos]
            .iter()
            .map(|bc| bc.ch)
            .collect();

        segments.push(WrappedSegment {
            content: segment_text,
            logical_start: current_char_pos,
            logical_end: segment_end_char_pos,
        });

        current_char_pos = segment_end_char_pos;
    }

    segments
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
        assert_eq!(cache.display_lines[0].content(), "Line 1");
        assert_eq!(cache.display_lines[1].content(), "Line 2");
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
        let pos = cache.logical_to_display_position(0, 0).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);

        // Position in middle should map to appropriate display line
        if let Some(pos) = cache.logical_to_display_position(0, 15) {
            assert!(pos.row > 0); // Should be on a wrapped line
            assert!(pos.col < 10); // Within the content width
        }
    }

    #[test]
    fn display_to_logical_position_should_work() {
        let lines = vec!["Hello world test".to_string()];
        let cache = build_display_cache(&lines, 10, true).unwrap();

        // First display line should map back to logical line 0
        let pos = cache.display_to_logical_position(0, 5).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn move_up_down_should_work() {
        let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        // Move down from first line
        let pos = cache.move_down(0, 3).unwrap();
        assert_eq!(pos.row, 1);
        assert_eq!(pos.col, 3);

        // Move up from second line
        let pos = cache.move_up(1, 3).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 3);
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
