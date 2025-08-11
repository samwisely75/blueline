//! # Mock I/O Implementations for Testing
//!
//! Provides mock implementations of EventStream and RenderStream traits
//! for testing without terminal dependencies.

use super::{EventStream, RenderStream, TerminalSize};
use anyhow::Result;
use crossterm::event::Event;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use vte;

// Use ansi_sequences module as ansi for cleaner code
use self::ansi_sequences as ansi;

/// Mock event stream for testing
///
/// Provides pre-programmed events that can be consumed by tests.
#[derive(Debug)]
pub struct MockEventStream {
    events: VecDeque<Event>,
    poll_always_true: bool,
}

impl MockEventStream {
    /// Create a new mock event stream with pre-programmed events
    pub fn new(events: Vec<Event>) -> Self {
        Self {
            events: events.into_iter().collect(),
            poll_always_true: true,
        }
    }

    /// Create an empty mock event stream
    pub fn empty() -> Self {
        Self {
            events: VecDeque::new(),
            poll_always_true: false,
        }
    }

    /// Set whether poll should always return true
    pub fn set_poll_behavior(&mut self, always_true: bool) {
        self.poll_always_true = always_true;
    }

    /// Add an event to the stream
    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }
}

impl EventStream for MockEventStream {
    fn poll(&mut self, _timeout: Duration) -> Result<bool> {
        Ok(self.poll_always_true || !self.events.is_empty())
    }

    fn read(&mut self) -> Result<Event> {
        self.events
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No events available"))
    }
}

/// Recorded render command for verification
#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    ClearScreen,
    MoveCursor(u16, u16),
    HideCursor,
    ShowCursor,
    GetSize,
    EnterAlternateScreen,
    LeaveAlternateScreen,
    EnableRawMode,
    DisableRawMode,
    Write(Vec<u8>),
    Flush,
}

/// Type alias for command history
type CommandHistory = Arc<Mutex<Vec<RenderCommand>>>;

/// Mock render stream for testing
///
/// Records all rendering commands for verification in tests.
pub struct MockRenderStream {
    commands: CommandHistory,
    buffer: Vec<u8>,
    terminal_size: TerminalSize,
    cursor_visible: bool,
    raw_mode: bool,
    alternate_screen: bool,
}

impl MockRenderStream {
    /// Create a new mock render stream
    pub fn new() -> Self {
        Self::with_size((80, 24))
    }

    /// Create a mock render stream with specific terminal size
    pub fn with_size(size: TerminalSize) -> Self {
        Self {
            commands: Arc::new(Mutex::new(Vec::new())),
            buffer: Vec::new(),
            terminal_size: size,
            cursor_visible: true,
            raw_mode: false,
            alternate_screen: false,
        }
    }

    /// Get recorded commands for verification
    pub fn get_commands(&self) -> Vec<RenderCommand> {
        self.commands.lock().unwrap().clone()
    }

    /// Clear recorded commands
    pub fn clear_commands(&mut self) {
        self.commands.lock().unwrap().clear();
    }

    /// Check if a specific command was recorded
    pub fn has_command(&self, command: &RenderCommand) -> bool {
        self.commands.lock().unwrap().contains(command)
    }

    /// Get the current buffer contents as a string
    pub fn get_buffer_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }

    fn record(&self, command: RenderCommand) {
        self.commands.lock().unwrap().push(command);
    }
}

impl Write for MockRenderStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.record(RenderCommand::Write(buf.to_vec()));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.record(RenderCommand::Flush);
        Ok(())
    }
}

impl RenderStream for MockRenderStream {
    fn clear_screen(&mut self) -> Result<()> {
        self.record(RenderCommand::ClearScreen);
        self.buffer.clear();
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.record(RenderCommand::MoveCursor(x, y));
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.record(RenderCommand::HideCursor);
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.record(RenderCommand::ShowCursor);
        self.cursor_visible = true;
        Ok(())
    }

    fn get_size(&self) -> Result<TerminalSize> {
        self.record(RenderCommand::GetSize);
        Ok(self.terminal_size)
    }

    fn enter_alternate_screen(&mut self) -> Result<()> {
        self.record(RenderCommand::EnterAlternateScreen);
        self.alternate_screen = true;
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> Result<()> {
        self.record(RenderCommand::LeaveAlternateScreen);
        self.alternate_screen = false;
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        self.record(RenderCommand::EnableRawMode);
        self.raw_mode = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        self.record(RenderCommand::DisableRawMode);
        self.raw_mode = false;
        Ok(())
    }
}

impl Default for MockEventStream {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for MockRenderStream {
    fn default() -> Self {
        Self::new()
    }
}

// ANSI escape sequence constants for VTE stream
#[allow(dead_code)]
mod ansi_sequences {
    // Screen control
    pub const CLEAR_SCREEN: &[u8] = b"\x1b[2J";
    pub const CURSOR_HOME: &[u8] = b"\x1b[H";

    // Cursor visibility
    pub const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
    pub const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

    // Alternate screen
    pub const ENTER_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049h";
    pub const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";

    // ANSI command characters for parsing
    pub const ESC: char = '\x1b';
    pub const CSI_START: char = '[';
    pub const CMD_CURSOR_POSITION: char = 'H';
    pub const CMD_CURSOR_POSITION_ALT: char = 'f';
    pub const CMD_CLEAR: char = 'J';
    pub const CMD_CLEAR_LINE: char = 'K';
    pub const CMD_CLEAR_ALL: &str = "2";

    // Control characters
    pub const NEWLINE: char = '\n';
    pub const CARRIAGE_RETURN: char = '\r';
    pub const PARAM_SEPARATOR: char = ';';
}

/// Terminal state information parsed from VTE
#[derive(Debug, Clone)]
pub struct TerminalStateInfo {
    pub grid: Vec<Vec<char>>,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub width: u16,
    pub height: u16,
}

/// VTE Performer that interprets escape sequences and builds terminal state
struct VtePerformer {
    grid: Vec<Vec<char>>,
    cursor_x: u16,
    cursor_y: u16,
    width: u16,
    height: u16,
    // Track SGR (Select Graphic Rendition) state for colors/styles
    current_fg: Option<u8>,
    current_bg: Option<u8>,
    bold: bool,
    reverse: bool,
}

impl VtePerformer {
    fn new(size: TerminalSize) -> Self {
        let (width, height) = size;
        Self {
            grid: vec![vec![' '; width as usize]; height as usize],
            cursor_x: 0,
            cursor_y: 0,
            width,
            height,
            current_fg: None,
            current_bg: None,
            bold: false,
            reverse: false,
        }
    }

    fn cursor_position(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }

    fn clear_screen(&mut self) {
        self.grid = vec![vec![' '; self.width as usize]; self.height as usize];
    }

    fn clear_line(&mut self, mode: u16) {
        if self.cursor_y >= self.height {
            return;
        }

        let row = self.cursor_y as usize;
        match mode {
            0 => {
                // Clear from cursor to end of line
                for x in self.cursor_x..self.width {
                    self.grid[row][x as usize] = ' ';
                }
            }
            1 => {
                // Clear from beginning of line to cursor
                for x in 0..=self.cursor_x {
                    self.grid[row][x as usize] = ' ';
                }
            }
            2 => {
                // Clear entire line
                self.grid[row] = vec![' '; self.width as usize];
            }
            _ => {}
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_x < self.width && self.cursor_y < self.height {
            self.grid[self.cursor_y as usize][self.cursor_x as usize] = c;
            self.cursor_x += 1;

            // Wrap to next line if needed
            if self.cursor_x >= self.width {
                self.cursor_x = 0;
                self.cursor_y = (self.cursor_y + 1).min(self.height - 1);
            }
        }
    }
}

impl vte::Perform for VtePerformer {
    fn print(&mut self, c: char) {
        // Regular printable character
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        // Control characters (C0 or C1)
        match byte {
            b'\n' => {
                // Line feed
                self.cursor_y = (self.cursor_y + 1).min(self.height - 1);
            }
            b'\r' => {
                // Carriage return
                self.cursor_x = 0;
            }
            b'\t' => {
                // Tab - move to next tab stop (every 8 columns)
                let next_tab = ((self.cursor_x / 8) + 1) * 8;
                self.cursor_x = next_tab.min(self.width - 1);
            }
            0x08 => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // Device control strings - not needed for basic terminal emulation
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // Operating system commands - not needed for basic terminal emulation
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        // Control Sequence Introducer commands - this is where most escape sequences are handled

        // Helper to get param value with default
        let get_param = |params: &vte::Params, idx: usize, default: u16| -> u16 {
            params
                .iter()
                .nth(idx)
                .and_then(|p| p.first())
                .copied()
                .unwrap_or(default)
        };

        match action {
            'H' | 'f' => {
                // CUP - Cursor Position (1-indexed)
                let row = get_param(params, 0, 1);
                let col = get_param(params, 1, 1);
                self.cursor_y = (row.saturating_sub(1)).min(self.height - 1);
                self.cursor_x = (col.saturating_sub(1)).min(self.width - 1);
            }
            'A' => {
                // CUU - Cursor Up
                let n = get_param(params, 0, 1);
                self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            'B' => {
                // CUD - Cursor Down
                let n = get_param(params, 0, 1);
                self.cursor_y = (self.cursor_y + n).min(self.height - 1);
            }
            'C' => {
                // CUF - Cursor Forward
                let n = get_param(params, 0, 1);
                self.cursor_x = (self.cursor_x + n).min(self.width - 1);
            }
            'D' => {
                // CUB - Cursor Back
                let n = get_param(params, 0, 1);
                self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            'J' => {
                // ED - Erase Display
                let mode = get_param(params, 0, 0);
                match mode {
                    0 => {
                        // Clear from cursor to end of screen
                        self.clear_line(0);
                        for y in (self.cursor_y + 1)..self.height {
                            self.grid[y as usize] = vec![' '; self.width as usize];
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of screen
                        self.clear_line(1);
                        for y in 0..self.cursor_y {
                            self.grid[y as usize] = vec![' '; self.width as usize];
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                // EL - Erase Line
                let mode = get_param(params, 0, 0);
                self.clear_line(mode);
            }
            'm' => {
                // SGR - Select Graphic Rendition (colors, bold, etc.)
                if params.is_empty() {
                    // Reset all attributes
                    self.current_fg = None;
                    self.current_bg = None;
                    self.bold = false;
                    self.reverse = false;
                } else {
                    for param in params.iter() {
                        match param.first() {
                            Some(&0) => {
                                // Reset
                                self.current_fg = None;
                                self.current_bg = None;
                                self.bold = false;
                                self.reverse = false;
                            }
                            Some(&1) => self.bold = true,
                            Some(&7) => self.reverse = true,
                            Some(&22) => self.bold = false,
                            Some(&27) => self.reverse = false,
                            Some(n @ 30..=37) => self.current_fg = Some((n - 30) as u8),
                            Some(n @ 40..=47) => self.current_bg = Some((n - 40) as u8),
                            Some(n @ 90..=97) => self.current_fg = Some((n - 90 + 8) as u8),
                            Some(n @ 100..=107) => self.current_bg = Some((n - 100 + 8) as u8),
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                // Unhandled CSI sequence - ignore for now
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences (without CSI) - not needed for basic terminal emulation
    }
}

// Type aliases to reduce complexity
type CapturedOutput = Arc<Mutex<Vec<u8>>>;
type CursorPosition = Arc<Mutex<(u16, u16)>>;

/// VTE-based render stream for integration testing
///
/// Captures terminal output and uses VTE parser to reconstruct terminal state.
/// This allows tests to verify the actual terminal display output.
#[derive(Debug)]
pub struct VteRenderStream {
    /// Raw captured terminal output
    captured: CapturedOutput,
    /// Terminal size for rendering
    terminal_size: TerminalSize,
    /// Cursor visibility state
    cursor_visible: bool,
    /// Raw mode state
    raw_mode: bool,
    /// Alternate screen state
    alternate_screen: bool,
    /// Current cursor position (x, y)
    cursor_position: CursorPosition,
}

impl VteRenderStream {
    /// Create a new VTE render stream
    pub fn new() -> Self {
        Self::with_size((80, 24))
    }

    /// Create a VTE render stream with specific terminal size
    pub fn with_size(size: TerminalSize) -> Self {
        Self {
            captured: Arc::new(Mutex::new(Vec::new())),
            terminal_size: size,
            cursor_visible: true,
            raw_mode: false,
            alternate_screen: false,
            cursor_position: Arc::new(Mutex::new((0, 0))),
        }
    }

    /// Get the raw captured output
    pub fn get_captured(&self) -> Vec<u8> {
        self.captured.lock().unwrap().clone()
    }

    /// Get captured output as string
    pub fn get_captured_string(&self) -> String {
        String::from_utf8_lossy(&self.get_captured()).to_string()
    }

    /// Clear captured output
    pub fn clear_captured(&self) {
        self.captured.lock().unwrap().clear();
    }

    /// Get current cursor position
    pub fn get_cursor_position(&self) -> (u16, u16) {
        *self.cursor_position.lock().unwrap()
    }

    /// Parse the captured output to reconstruct terminal state
    /// Returns a properly parsed terminal state using VTE
    pub fn parse_terminal_state(&self) -> TerminalStateInfo {
        let captured = self.get_captured();
        let mut parser = vte::Parser::new();
        let mut performer = VtePerformer::new(self.terminal_size);

        // Feed all captured bytes through the VTE parser
        for byte in captured {
            parser.advance(&mut performer, byte);
        }

        // Update our stored cursor position
        *self.cursor_position.lock().unwrap() = performer.cursor_position();

        TerminalStateInfo {
            grid: performer.grid,
            cursor_x: performer.cursor_x,
            cursor_y: performer.cursor_y,
            width: performer.width,
            height: performer.height,
        }
    }

    /// Get a 2D grid of characters (backwards compatibility)
    pub fn get_grid(&self) -> Vec<Vec<char>> {
        self.parse_terminal_state().grid
    }

    /// Get a specific line from the terminal as a string
    pub fn get_line(&self, line_num: usize) -> String {
        let state = self.parse_terminal_state();
        if line_num < state.grid.len() {
            state.grid[line_num]
                .iter()
                .collect::<String>()
                .trim_end()
                .to_string()
        } else {
            String::new()
        }
    }

    /// Check if the terminal contains specific text
    pub fn contains_text(&self, text: &str) -> bool {
        let state = self.parse_terminal_state();
        for row in state.grid {
            let line: String = row.iter().collect();
            if line.contains(text) {
                return true;
            }
        }
        false
    }
}

impl Write for VteRenderStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.captured.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl RenderStream for VteRenderStream {
    fn clear_screen(&mut self) -> Result<()> {
        // Emit ANSI clear screen sequence
        self.write_all(ansi::CLEAR_SCREEN)?;
        self.write_all(ansi::CURSOR_HOME)?; // Move cursor to home
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        // Emit ANSI cursor position sequence (1-indexed)
        let seq = format!("\x1b[{};{}H", y + 1, x + 1);
        self.write_all(seq.as_bytes())?;
        *self.cursor_position.lock().unwrap() = (x, y);
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.write_all(ansi::HIDE_CURSOR)?;
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.write_all(ansi::SHOW_CURSOR)?;
        self.cursor_visible = true;
        Ok(())
    }

    fn get_size(&self) -> Result<TerminalSize> {
        Ok(self.terminal_size)
    }

    fn enter_alternate_screen(&mut self) -> Result<()> {
        self.write_all(ansi::ENTER_ALTERNATE_SCREEN)?;
        self.alternate_screen = true;
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> Result<()> {
        self.write_all(ansi::LEAVE_ALTERNATE_SCREEN)?;
        self.alternate_screen = false;
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = false;
        Ok(())
    }
}

impl Default for VteRenderStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn mock_event_stream_should_provide_events() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())),
            Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty())),
        ];

        let mut stream = MockEventStream::new(events);

        // Poll should return true when events available
        assert!(stream.poll(Duration::from_millis(0)).unwrap());

        // Read first event
        let event = stream.read().unwrap();
        if let Event::Key(key) = event {
            assert_eq!(key.code, KeyCode::Char('a'));
        } else {
            panic!("Expected key event");
        }

        // Still have events
        assert!(stream.poll(Duration::from_millis(0)).unwrap());

        // Read second event
        let event = stream.read().unwrap();
        if let Event::Key(key) = event {
            assert_eq!(key.code, KeyCode::Char('b'));
        } else {
            panic!("Expected key event");
        }

        // No more events (unless poll_always_true is set)
        assert!(!stream.poll(Duration::from_millis(0)).unwrap() || stream.poll_always_true);
    }

    #[test]
    fn mock_render_stream_should_record_commands() {
        let mut stream = MockRenderStream::new();

        // Perform various operations
        stream.clear_screen().unwrap();
        stream.move_cursor(10, 20).unwrap();
        stream.hide_cursor().unwrap();
        stream.write_all(b"Hello").unwrap();
        stream.flush().unwrap();

        // Verify commands were recorded
        let commands = stream.get_commands();
        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], RenderCommand::ClearScreen);
        assert_eq!(commands[1], RenderCommand::MoveCursor(10, 20));
        assert_eq!(commands[2], RenderCommand::HideCursor);
        assert_eq!(commands[3], RenderCommand::Write(b"Hello".to_vec()));
        assert_eq!(commands[4], RenderCommand::Flush);

        // Verify buffer contents
        assert_eq!(stream.get_buffer_string(), "Hello");
    }

    #[test]
    fn mock_render_stream_should_track_state() {
        let mut stream = MockRenderStream::with_size((120, 40));

        // Check initial state
        assert_eq!(stream.get_size().unwrap(), (120, 40));
        assert!(stream.cursor_visible);
        assert!(!stream.raw_mode);
        assert!(!stream.alternate_screen);

        // Change state
        stream.hide_cursor().unwrap();
        stream.enable_raw_mode().unwrap();
        stream.enter_alternate_screen().unwrap();

        // Verify state changes
        assert!(!stream.cursor_visible);
        assert!(stream.raw_mode);
        assert!(stream.alternate_screen);
    }
}
