//! # Display Buffer Cache - High-Performance Wrapped Text Rendering
//!
//! This module implements a display buffer cache system that pre-calculates
//! wrapped text layouts for optimal cursor positioning and scrolling performance.
//!
//! ## Architecture
//!
//! - **DisplayCache**: Pre-calculated display lines with metadata
//! - **Threaded Updates**: Background recalculation for Request pane edits
//! - **Invalidation Strategy**: Smart cache management
//! - **O(1) Operations**: Cursor positioning becomes array indexing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Type alias for position tuples to reduce complexity
pub type Position = (usize, usize);

/// Type alias for logical-to-display line mapping
pub type LogicalToDisplayMap = HashMap<usize, Vec<usize>>;

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
        }
    }

    /// Check if cache is valid for given content and width
    pub fn is_valid_for(&self, content_hash: u64, width: usize) -> bool {
        self.is_valid && self.content_hash == content_hash && self.content_width == width
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
    ) -> Option<Position> {
        if !self.is_valid {
            return None;
        }

        // Find display lines for this logical line
        let display_indices = self.logical_to_display.get(&logical_line)?;

        for &display_idx in display_indices {
            if let Some(display_line) = self.display_lines.get(display_idx) {
                // Check if cursor falls within this display line segment
                if logical_col >= display_line.logical_start_col
                    && logical_col < display_line.logical_end_col
                {
                    let display_col = logical_col - display_line.logical_start_col;
                    return Some((display_idx, display_col));
                }
                // Handle end-of-line case
                if logical_col == display_line.logical_end_col
                    && display_line.logical_end_col > display_line.logical_start_col
                {
                    let display_col = display_line.logical_end_col - display_line.logical_start_col;
                    return Some((display_idx, display_col));
                }
            }
        }

        // Fallback to last display line of this logical line
        if let Some(&last_display_idx) = display_indices.last() {
            if let Some(display_line) = self.display_lines.get(last_display_idx) {
                let display_col = (display_line.content.chars().count())
                    .min(logical_col - display_line.logical_start_col);
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
    ) -> Option<Position> {
        if !self.is_valid {
            return None;
        }

        let display_info = self.display_lines.get(display_line)?;
        let logical_line = display_info.logical_line;
        let logical_col = display_info.logical_start_col + display_col;

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
    pub fn move_up(&self, current_display_line: usize, desired_col: usize) -> Option<Position> {
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
    pub fn move_down(&self, current_display_line: usize, desired_col: usize) -> Option<Position> {
        if !self.is_valid || current_display_line >= self.display_lines.len().saturating_sub(1) {
            return None;
        }

        let target_display_line = current_display_line + 1;
        let target_display_info = self.display_lines.get(target_display_line)?;

        // Try to maintain column position, but clamp to line length
        let target_col = desired_col.min(target_display_info.content.chars().count());

        Some((target_display_line, target_col))
    }
}

impl Default for DisplayCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe cache manager for background updates
#[derive(Debug)]
pub struct CacheManager {
    /// Cached display buffer for Response pane (read-only, updated infrequently)
    pub response_cache: Arc<Mutex<DisplayCache>>,
    /// Cached display buffer for Request pane (editable, updated frequently)
    pub request_cache: Arc<Mutex<DisplayCache>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self {
            response_cache: Arc::new(Mutex::new(DisplayCache::new())),
            request_cache: Arc::new(Mutex::new(DisplayCache::new())),
        }
    }

    /// Update response cache (synchronous - infrequent updates)
    pub fn update_response_cache(
        &self,
        lines: &[String],
        content_width: usize,
    ) -> anyhow::Result<()> {
        let content_hash = calculate_content_hash(lines);

        {
            let cache = self.response_cache.lock().unwrap();
            if cache.is_valid_for(content_hash, content_width) {
                return Ok(()); // Cache is still valid
            }
        }

        let new_cache = build_display_cache(lines, content_width, content_hash)?;

        {
            let mut cache = self.response_cache.lock().unwrap();
            *cache = new_cache;
        }

        Ok(())
    }

    /// Update request cache (asynchronous - frequent updates from editing)
    pub fn update_request_cache_async(&self, lines: Vec<String>, content_width: usize) {
        let content_hash = calculate_content_hash(&lines);

        // Check if update is needed
        {
            let cache = self.request_cache.lock().unwrap();
            if cache.is_valid_for(content_hash, content_width) {
                return; // Cache is still valid
            }
        }

        // Clone the Arc for the thread
        let cache_ref = Arc::clone(&self.request_cache);

        // Spawn background thread for cache calculation
        thread::spawn(move || {
            match build_display_cache(&lines, content_width, content_hash) {
                Ok(new_cache) => {
                    if let Ok(mut cache) = cache_ref.lock() {
                        // Only update if this calculation is still relevant
                        if !cache.is_valid_for(content_hash, content_width) {
                            *cache = new_cache;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to build display cache: {}", e);
                }
            }
        });
    }

    /// Update request cache (synchronous - for immediate rendering)
    pub fn update_request_cache_sync(
        &self,
        lines: &[String],
        content_width: usize,
    ) -> anyhow::Result<()> {
        let content_hash = calculate_content_hash(lines);

        {
            let cache = self.request_cache.lock().unwrap();
            if cache.is_valid_for(content_hash, content_width) {
                return Ok(()); // Cache is still valid
            }
        }

        let new_cache = build_display_cache(lines, content_width, content_hash)?;

        {
            let mut cache = self.request_cache.lock().unwrap();
            *cache = new_cache;
        }

        Ok(())
    }

    /// Get a copy of the response cache for safe access
    pub fn get_response_cache(&self) -> DisplayCache {
        self.response_cache.lock().unwrap().clone()
    }

    /// Get a copy of the request cache for safe access
    pub fn get_request_cache(&self) -> DisplayCache {
        self.request_cache.lock().unwrap().clone()
    }

    /// Invalidate all caches (e.g., on window resize)
    pub fn invalidate_all(&self) {
        {
            let mut cache = self.response_cache.lock().unwrap();
            cache.invalidate();
        }
        {
            let mut cache = self.request_cache.lock().unwrap();
            cache.invalidate();
        }
    }
}

impl Default for CacheManager {
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
fn build_display_cache(
    lines: &[String],
    content_width: usize,
    content_hash: u64,
) -> anyhow::Result<DisplayCache> {
    let mut display_lines = Vec::new();
    let mut logical_to_display = HashMap::new();

    for (logical_idx, line) in lines.iter().enumerate() {
        let wrapped_segments = wrap_line_with_positions(line, content_width);
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

/// Wrap a line into segments based on content width (vim-style) - legacy interface
fn wrap_line(line: &str, content_width: usize) -> Vec<String> {
    wrap_line_with_positions(line, content_width)
        .into_iter()
        .map(|segment| segment.content)
        .collect()
}

/// Calculate a simple hash of the content for invalidation detection
fn calculate_content_hash(lines: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    lines.hash(&mut hasher);
    hasher.finish()
}

/// Debug function to dump cache information to a file
impl DisplayCache {
    pub fn debug_dump(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "=== DISPLAY CACHE DEBUG DUMP ===")?;
        writeln!(writer, "Valid: {}", self.is_valid)?;
        writeln!(writer, "Content Width: {}", self.content_width)?;
        writeln!(writer, "Content Hash: {}", self.content_hash)?;
        writeln!(writer, "Total Display Lines: {}", self.total_display_lines)?;
        writeln!(writer)?;

        writeln!(writer, "=== DISPLAY LINES ===")?;
        for (idx, display_line) in self.display_lines.iter().enumerate() {
            writeln!(
                writer,
                "Display[{}]: logical={}, start_col={}, end_col={}, continuation={}, content='{}'",
                idx,
                display_line.logical_line,
                display_line.logical_start_col,
                display_line.logical_end_col,
                display_line.is_continuation,
                display_line.content
            )?;
        }

        writeln!(writer)?;
        writeln!(writer, "=== LOGICAL TO DISPLAY MAPPING ===")?;
        for (logical_line, display_indices) in &self.logical_to_display {
            writeln!(
                writer,
                "Logical[{}] -> Display{:?}",
                logical_line, display_indices
            )?;
        }

        Ok(())
    }
}

/// Debug function to test wrapping with detailed output
pub fn debug_wrap_line(line: &str, content_width: usize) -> Vec<String> {
    println!("DEBUG WRAP: input='{}', width={}", line, content_width);
    let result = wrap_line(line, content_width);
    for (i, segment) in result.iter().enumerate() {
        println!(
            "  Segment[{}]: '{}' (len={})",
            i,
            segment,
            segment.chars().count()
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_cache_should_calculate_wrapped_lines_correctly() {
        let lines = vec![
            "This is a very long line that should wrap across multiple display lines".to_string(),
            "Short line".to_string(),
        ];
        let content_width = 20;
        let content_hash = calculate_content_hash(&lines);

        let cache = build_display_cache(&lines, content_width, content_hash).unwrap();

        assert!(cache.is_valid);
        assert_eq!(cache.content_width, content_width);
        assert!(cache.display_lines.len() > 2); // Should have more display lines than logical lines

        // First logical line should map to multiple display lines
        let display_indices = cache.logical_to_display.get(&0).unwrap();
        assert!(display_indices.len() > 1);

        // Second logical line should map to one display line
        let display_indices = cache.logical_to_display.get(&1).unwrap();
        assert_eq!(display_indices.len(), 1);
    }

    #[test]
    fn logical_to_display_position_should_work_correctly() {
        let lines = vec!["This is a very long line that should wrap".to_string()];
        let content_width = 20;
        let content_hash = calculate_content_hash(&lines);

        let cache = build_display_cache(&lines, content_width, content_hash).unwrap();

        // Position at start of line
        let (display_line, display_col) = cache.logical_to_display_position(0, 0).unwrap();
        assert_eq!(display_line, 0);
        assert_eq!(display_col, 0);

        // Position in middle of line (should be on second display line)
        let (display_line, display_col) = cache.logical_to_display_position(0, 25).unwrap();
        assert!(display_line > 0);
        assert!(display_col < content_width);
    }

    #[test]
    fn move_up_down_should_work_correctly() {
        let lines = vec![
            "First line".to_string(),
            "Second line".to_string(),
            "Third line".to_string(),
        ];
        let content_width = 80;
        let content_hash = calculate_content_hash(&lines);

        let cache = build_display_cache(&lines, content_width, content_hash).unwrap();

        // Move down from first line
        let (new_display_line, new_col) = cache.move_down(0, 5).unwrap();
        assert_eq!(new_display_line, 1);
        assert_eq!(new_col, 5);

        // Move up from second line
        let (new_display_line, new_col) = cache.move_up(1, 5).unwrap();
        assert_eq!(new_display_line, 0);
        assert_eq!(new_col, 5);
    }

    #[test]
    fn cache_invalidation_should_work_correctly() {
        let lines = vec!["Test line".to_string()];
        let content_hash = calculate_content_hash(&lines);

        let cache = build_display_cache(&lines, 80, content_hash).unwrap();
        assert!(cache.is_valid_for(content_hash, 80));

        // Different content should invalidate
        let different_lines = vec!["Different line".to_string()];
        let different_hash = calculate_content_hash(&different_lines);
        assert!(!cache.is_valid_for(different_hash, 80));

        // Different width should invalidate
        assert!(!cache.is_valid_for(content_hash, 60));
    }

    #[test]
    fn wrap_line_should_break_long_lines_correctly() {
        // Test with a long line that should wrap
        let long_line = "This is a very long line that should definitely wrap across multiple segments when the content width is small";
        let content_width = 20;

        let segments = wrap_line(long_line, content_width);

        // Should have multiple segments
        assert!(
            segments.len() > 1,
            "Expected multiple segments, got: {:?}",
            segments
        );

        // Each segment should be <= content_width
        for (i, segment) in segments.iter().enumerate() {
            assert!(
                segment.chars().count() <= content_width,
                "Segment {} '{}' has {} chars, expected <= {}",
                i,
                segment,
                segment.chars().count(),
                content_width
            );
        }

        // Reconstruct original (approximately - spaces may be trimmed)
        let reconstructed = segments.join(" ");
        assert!(!reconstructed.is_empty());

        // Test with exact width
        let exact_line = "12345678901234567890"; // exactly 20 chars
        let exact_segments = wrap_line(exact_line, 20);
        assert_eq!(exact_segments.len(), 1);
        assert_eq!(exact_segments[0], exact_line);

        // Test with slightly longer
        let longer_line = "123456789012345678901"; // 21 chars
        let longer_segments = wrap_line(longer_line, 20);
        assert_eq!(longer_segments.len(), 2);
        assert_eq!(longer_segments[0].chars().count(), 20);
        assert_eq!(longer_segments[1].chars().count(), 1);
    }

    #[test]
    fn debug_wrap_realistic_scenarios() {
        // Test realistic HTTP request line that might be on line 30
        let http_line = "POST /api/users/12345/profile HTTP/1.1";
        println!("Testing: '{}'", http_line);
        let segments = debug_wrap_line(http_line, 30);
        assert!(!segments.is_empty());

        // Test long authorization header
        let auth_line = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        println!("Testing long auth: '{}'", auth_line);
        let segments = debug_wrap_line(auth_line, 60);
        assert!(
            segments.len() > 1,
            "Long auth header should wrap into multiple segments"
        );

        // Test edge case with exact boundary
        let boundary_line = "Content-Type: application/json; charset=utf-8";
        println!("Testing boundary: '{}'", boundary_line);
        let segments = debug_wrap_line(boundary_line, 45);
        println!("Boundary result: {:?}", segments);
    }
}
