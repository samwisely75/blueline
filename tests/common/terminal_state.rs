use vte::{Params, Perform};

/// Represents the actual terminal screen state reconstructed from escape sequences
#[derive(Debug, Clone)]
pub struct TerminalState {
    pub grid: Vec<Vec<char>>,
    pub cursor: (usize, usize), // (row, col)
    pub width: usize,
    pub height: usize,
    pub cursor_visible: bool,
    // Track rendering statistics
    pub full_redraws: usize,
    pub partial_redraws: usize,
    pub cursor_updates: usize,
    pub clear_screen_count: usize,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

impl TerminalState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![' '; width]; height],
            cursor: (0, 0),
            width,
            height,
            cursor_visible: true,
            full_redraws: 0,
            partial_redraws: 0,
            cursor_updates: 0,
            clear_screen_count: 0,
        }
    }

    /// Get text content at a specific region
    pub fn get_text_at_region(
        &self,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> String {
        let mut result = String::new();
        for row in start_row..=end_row.min(self.height - 1) {
            for col in start_col..=end_col.min(self.width - 1) {
                result.push(self.grid[row][col]);
            }
            if row < end_row {
                result.push('\n');
            }
        }
        result.trim_end().to_string()
    }

    /// Get full screen content as string
    pub fn get_full_text(&self) -> String {
        self.get_text_at_region(0, 0, self.height - 1, self.width - 1)
    }

    /// Check if text exists anywhere on screen
    pub fn contains_text(&self, text: &str) -> bool {
        self.get_full_text().contains(text)
    }

    /// Get character at specific position
    pub fn get_char_at(&self, row: usize, col: usize) -> Option<char> {
        if row < self.height && col < self.width {
            Some(self.grid[row][col])
        } else {
            None
        }
    }

    /// Clear screen
    fn clear_screen(&mut self) {
        for row in &mut self.grid {
            row.fill(' ');
        }
        self.clear_screen_count += 1;
    }

    /// Clear line
    fn clear_line(&mut self, row: usize) {
        if row < self.height {
            self.grid[row].fill(' ');
        }
    }
}

/// Implementation of VTE Perform trait to handle terminal escape sequences
impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        // Place character at cursor position and advance cursor
        if self.cursor.0 < self.height && self.cursor.1 < self.width {
            self.grid[self.cursor.0][self.cursor.1] = c;
            self.cursor.1 += 1;

            // Wrap to next line if we hit the end
            if self.cursor.1 >= self.width {
                self.cursor.1 = 0;
                self.cursor.0 += 1;

                // Ensure cursor doesn't go beyond the bottom of the screen
                if self.cursor.0 >= self.height {
                    self.cursor.0 = self.height - 1;
                }
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line feed - move cursor down
                self.cursor.0 = (self.cursor.0 + 1).min(self.height - 1);
            }
            b'\r' => {
                // Carriage return - move cursor to start of line
                self.cursor.1 = 0;
            }
            b'\t' => {
                // Tab - move to next tab stop (every 8 characters)
                let next_tab = ((self.cursor.1 / 8) + 1) * 8;
                self.cursor.1 = next_tab.min(self.width - 1);
            }
            _ => {
                // Handle other control characters as needed
            }
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'H' | 'f' => {
                // Cursor Position (CUP) - Move cursor to specific position
                let row = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                let col = params
                    .iter()
                    .nth(1)
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor = (
                    row.saturating_sub(1).min(self.height - 1),
                    col.saturating_sub(1).min(self.width - 1),
                );
                self.cursor_updates += 1;
            }
            'A' => {
                // Cursor Up (CUU)
                let count = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor.0 = self.cursor.0.saturating_sub(count);
                self.cursor_updates += 1;
            }
            'B' => {
                // Cursor Down (CUD)
                let count = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor.0 = (self.cursor.0 + count).min(self.height - 1);
                self.cursor_updates += 1;
            }
            'C' => {
                // Cursor Forward (CUF)
                let count = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor.1 = (self.cursor.1 + count).min(self.width - 1);
                self.cursor_updates += 1;
            }
            'D' => {
                // Cursor Backward (CUB)
                let count = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor.1 = self.cursor.1.saturating_sub(count);
                self.cursor_updates += 1;
            }
            'J' => {
                // Erase in Display (ED)
                let param = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                match param {
                    0 => {
                        // Clear from cursor to end of screen
                        if self.cursor.0 < self.height {
                            for col in self.cursor.1..self.width {
                                self.grid[self.cursor.0][col] = ' ';
                            }
                        }
                        for row in (self.cursor.0 + 1)..self.height {
                            self.grid[row].fill(' ');
                        }
                        self.partial_redraws += 1;
                    }
                    1 => {
                        // Clear from beginning of screen to cursor
                        for row in 0..self.cursor.0 {
                            self.grid[row].fill(' ');
                        }
                        if self.cursor.0 < self.height {
                            for col in 0..=self.cursor.1.min(self.width - 1) {
                                self.grid[self.cursor.0][col] = ' ';
                            }
                        }
                        self.partial_redraws += 1;
                    }
                    2 => {
                        // Clear entire screen
                        self.clear_screen();
                        self.full_redraws += 1;
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line (EL)
                let param = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                match param {
                    0 => {
                        // Clear from cursor to end of line
                        if self.cursor.0 < self.height {
                            for col in self.cursor.1..self.width {
                                self.grid[self.cursor.0][col] = ' ';
                            }
                        }
                    }
                    1 => {
                        // Clear from beginning of line to cursor
                        if self.cursor.0 < self.height {
                            for col in 0..=self.cursor.1.min(self.width - 1) {
                                self.grid[self.cursor.0][col] = ' ';
                            }
                        }
                    }
                    2 => {
                        // Clear entire line
                        self.clear_line(self.cursor.0);
                    }
                    _ => {}
                }
                self.partial_redraws += 1;
            }
            'm' => {
                // SGR (Select Graphic Rendition) - colors and attributes
                // For now, we ignore color/attribute changes but could track them
            }
            'h' | 'l' => {
                // Set/Reset Mode
                // Check for cursor visibility (DECTCEM - ?25h/?25l)
                if !_intermediates.is_empty() && _intermediates[0] == b'?' {
                    if let Some(param) = params.iter().next().and_then(|p| p.first()) {
                        if *param == 25 {
                            self.cursor_visible = action == 'h';
                        }
                    }
                }
            }
            'G' => {
                // Cursor Horizontal Absolute (CHA) - Move cursor to specific column
                let col = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.cursor.1 = col.saturating_sub(1).min(self.width - 1);
                self.cursor_updates += 1;
            }
            _ => {
                // Ignore unhandled sequences for now
            }
        }
    }
}
