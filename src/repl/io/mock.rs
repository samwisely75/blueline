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

/// Mock event stream for testing
///
/// Provides pre-programmed events that can be consumed by tests.
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
