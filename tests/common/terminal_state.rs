//! Terminal state parsing and assertions
//!
//! This module provides utilities for parsing VTE output and making assertions
//! about terminal content in integration tests.

#![allow(dead_code)]

use blueline::repl::io::mock::{TerminalStateInfo, VteRenderStream};
use tracing::{debug, trace};

/// Represents the parsed terminal state for test assertions
#[derive(Debug, Clone)]
pub struct TerminalState {
    /// The terminal grid as parsed by VTE
    pub grid: Vec<Vec<char>>,
    /// Current cursor position (column, row)
    pub cursor_position: (u16, u16),
    /// Terminal dimensions
    pub width: u16,
    pub height: u16,
}

impl TerminalState {
    /// Create a new terminal state from a VteRenderStream
    pub fn from_render_stream(stream: &VteRenderStream) -> Self {
        let state_info = stream.parse_terminal_state();
        Self::from_state_info(state_info)
    }

    /// Create from TerminalStateInfo
    pub fn from_state_info(info: TerminalStateInfo) -> Self {
        Self {
            grid: info.grid,
            cursor_position: (info.cursor_x, info.cursor_y),
            width: info.width,
            height: info.height,
        }
    }

    /// Create an empty default terminal state
    pub fn default() -> Self {
        Self {
            grid: vec![vec![' '; 80]; 24],
            cursor_position: (0, 0),
            width: 80,
            height: 24,
        }
    }

    /// Get the visible text content (non-empty lines)
    pub fn get_visible_text(&self) -> Vec<String> {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }

    /// Get a specific line of text (0-indexed)
    pub fn get_line(&self, line_num: usize) -> Option<String> {
        self.grid
            .get(line_num)
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
    }

    /// Check if the terminal contains specific text
    pub fn contains(&self, text: &str) -> bool {
        for row in &self.grid {
            let line: String = row.iter().collect();
            if line.contains(text) {
                return true;
            }
        }
        false
    }

    /// Find the row containing specific text
    pub fn find_row(&self, text: &str) -> Option<usize> {
        for (idx, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().collect();
            if line.contains(text) {
                return Some(idx);
            }
        }
        None
    }

    /// Get the text at a specific region
    pub fn get_region(
        &self,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> String {
        let mut result = String::new();

        for row_idx in start_row..=end_row {
            if let Some(row) = self.grid.get(row_idx) {
                let start = if row_idx == start_row { start_col } else { 0 };
                let end = if row_idx == end_row {
                    end_col
                } else {
                    row.len()
                };

                for ch in row.iter().take(end.min(row.len())).skip(start) {
                    result.push(*ch);
                }

                if row_idx < end_row {
                    result.push('\n');
                }
            }
        }

        result
    }

    /// Check if cursor is at a specific position
    pub fn cursor_at(&self, col: u16, row: u16) -> bool {
        self.cursor_position == (col, row)
    }

    /// Get the character at cursor position
    pub fn char_at_cursor(&self) -> Option<char> {
        let (col, row) = self.cursor_position;
        self.grid
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
            .copied()
    }

    /// Assert that a specific line contains text
    pub fn assert_line_contains(&self, line_num: usize, expected: &str) {
        if let Some(line) = self.get_line(line_num) {
            assert!(
                line.contains(expected),
                "Line {line_num} does not contain '{expected}'. Actual: '{line}'"
            );
        } else {
            panic!(
                "Line {} does not exist (terminal has {} lines)",
                line_num,
                self.grid.len()
            );
        }
    }

    /// Assert cursor position
    pub fn assert_cursor_at(&self, expected_col: u16, expected_row: u16) {
        assert_eq!(
            self.cursor_position,
            (expected_col, expected_row),
            "Cursor is at ({}, {}), expected ({}, {})",
            self.cursor_position.0,
            self.cursor_position.1,
            expected_col,
            expected_row
        );
    }

    /// Get the prompt line (usually the last non-empty line)
    pub fn get_prompt_line(&self) -> Option<String> {
        // Search from bottom up for the first non-empty line
        for row in self.grid.iter().rev() {
            let line: String = row.iter().collect::<String>().trim_end().to_string();
            if !line.is_empty() {
                return Some(line);
            }
        }
        None
    }

    /// Debug helper: log the terminal grid with cursor position using tracing
    pub fn debug_print(&self) {
        debug!("Terminal State ({}x{}):", self.width, self.height);
        debug!(
            "Cursor at: ({}, {})",
            self.cursor_position.0, self.cursor_position.1
        );

        // Calculate the width needed for line numbers
        let max_line_num = self.grid.len();
        let line_num_width = max_line_num.to_string().len();

        // Log non-empty lines or lines with cursor
        for (row_idx, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().collect();
            let trimmed = line.trim_end();
            if !trimmed.is_empty() || row_idx == self.cursor_position.1 as usize {
                trace!("{row_idx:line_num_width$}: |{trimmed}|");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_parsing() {
        // Create a simple grid for testing
        let mut grid = vec![vec![' '; 80]; 24];

        // Add some text
        let text = "Hello, World!";
        for (i, ch) in text.chars().enumerate() {
            grid[0][i] = ch;
        }

        let state = TerminalState {
            grid,
            cursor_position: (13, 0),
            width: 80,
            height: 24,
        };

        // Test contains
        assert!(state.contains("Hello"));
        assert!(state.contains("World"));
        assert!(!state.contains("Goodbye"));

        // Test get_line
        assert_eq!(state.get_line(0), Some("Hello, World!".to_string()));
        assert_eq!(state.get_line(1), Some("".to_string()));

        // Test find_row
        assert_eq!(state.find_row("Hello"), Some(0));
        assert_eq!(state.find_row("Goodbye"), None);
    }
}
