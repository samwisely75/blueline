//! Testing infrastructure for REPL components
//!
//! Provides utilities for testing terminal UI components without requiring
//! actual terminal interaction, enabling automated testing of REPL functionality.

use crate::repl::model::{AppState, Pane};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::io::{self, Write};

/// Mock writer that captures output instead of writing to terminal
/// This allows testing terminal output without actual terminal interaction
#[derive(Default)]
pub struct MockWriter {
    pub output: Vec<u8>,
}

impl Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // No-op for mock writer
        Ok(())
    }
}

impl MockWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_output(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }
}

/// Mock event source for simulating keyboard input
/// Simulates crossterm events for testing without actual keyboard input
pub struct MockEventSource {
    events: VecDeque<Event>,
}

impl MockEventSource {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Add a key event to the mock event queue
    pub fn add_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        let event = Event::Key(KeyEvent::new(key_code, modifiers));
        self.events.push_back(event);
    }

    /// Add multiple character inputs as key events
    pub fn add_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.add_key(KeyCode::Char(ch), KeyModifiers::NONE);
        }
    }

    /// Add common key combinations
    pub fn add_escape(&mut self) {
        self.add_key(KeyCode::Esc, KeyModifiers::NONE);
    }

    pub fn add_enter(&mut self) {
        self.add_key(KeyCode::Enter, KeyModifiers::NONE);
    }

    pub fn add_ctrl_c(&mut self) {
        self.add_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
    }

    pub fn add_colon(&mut self) {
        self.add_key(KeyCode::Char(':'), KeyModifiers::NONE);
    }

    /// Get the next event from the queue
    pub fn next_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Check if there are remaining events
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Get count of remaining events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Helper for creating common REPL test scenarios
pub struct ReplTestHelper {
    pub model: AppState,
    pub mock_writer: MockWriter,
    pub event_source: MockEventSource,
}

impl ReplTestHelper {
    /// Create a new test helper with default configuration
    pub fn new() -> Self {
        Self {
            model: AppState::new((80, 24), false), // Default terminal size, no verbose
            mock_writer: MockWriter::new(),
            event_source: MockEventSource::new(),
        }
    }

    /// Create test helper with specific profile configuration
    pub fn with_profile(endpoint: &str, name: &str) -> Self {
        let mut model = AppState::new((80, 24), false);
        // Note: Profile configuration would need to be added to AppState
        // For now, just use default state
        let _ = (endpoint, name); // Suppress unused warnings

        Self {
            model,
            mock_writer: MockWriter::new(),
            event_source: MockEventSource::new(),
        }
    }

    /// Add a sequence of text input to the event source
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.event_source.add_text(text);
        self
    }

    /// Add enter key to the event source
    pub fn press_enter(&mut self) -> &mut Self {
        self.event_source.add_enter();
        self
    }

    /// Add escape key to the event source
    pub fn press_escape(&mut self) -> &mut Self {
        self.event_source.add_escape();
        self
    }

    /// Add colon command sequence
    pub fn enter_colon_command(&mut self, command: &str) -> &mut Self {
        self.event_source.add_colon();
        self.event_source.add_text(command);
        self.event_source.add_enter();
        self
    }

    /// Switch to insert mode and type text
    pub fn insert_text(&mut self, text: &str) -> &mut Self {
        self.event_source
            .add_key(KeyCode::Char('i'), KeyModifiers::NONE);
        self.event_source.add_text(text);
        self.event_source.add_escape(); // Exit insert mode
        self
    }

    /// Get the current request pane content
    pub fn get_request_content(&self) -> String {
        self.model.request_buffer.lines.join("\n")
    }

    /// Get the current response pane content
    pub fn get_response_content(&self) -> String {
        if let Some(ref response) = self.model.response_buffer {
            response.lines.join("\n")
        } else {
            String::new()
        }
    }

    /// Get the current active pane
    pub fn get_active_pane(&self) -> Pane {
        self.model.current_pane
    }

    /// Clear the mock writer output
    pub fn clear_output(&mut self) {
        self.mock_writer.clear();
    }

    /// Get the captured output from mock writer
    pub fn get_output(&self) -> String {
        self.mock_writer.get_output()
    }

    /// Consume the next event from the event source
    pub fn next_event(&mut self) -> Option<Event> {
        self.event_source.next_event()
    }

    /// Check if there are more events to process
    pub fn has_events(&self) -> bool {
        self.event_source.has_events()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_writer_captures_output() {
        let mut writer = MockWriter::new();
        write!(writer, "Hello, World!").unwrap();
        assert_eq!(writer.get_output(), "Hello, World!");
    }

    #[test]
    fn test_mock_event_source_adds_events() {
        let mut source = MockEventSource::new();
        source.add_text("hello");
        assert_eq!(source.event_count(), 5); // 5 characters

        // Consume one event
        let event = source.next_event().unwrap();
        if let Event::Key(key_event) = event {
            assert_eq!(key_event.code, KeyCode::Char('h'));
        }
        assert_eq!(source.event_count(), 4); // 4 remaining
    }

    #[test]
    fn test_repl_test_helper_basic_setup() {
        let helper = ReplTestHelper::new();
        assert_eq!(helper.get_active_pane(), Pane::Request);
        assert_eq!(helper.get_request_content(), "");
        assert_eq!(helper.get_response_content(), "");
    }

    #[test]
    fn test_repl_test_helper_with_profile() {
        let helper = ReplTestHelper::with_profile("https://api.example.com", "test");
        // Model should be configured with the profile
        // This would need specific model methods to verify
    }

    #[test]
    fn test_event_source_key_combinations() {
        let mut source = MockEventSource::new();
        source.add_ctrl_c();
        source.add_colon();
        source.add_enter();
        source.add_escape();

        assert_eq!(source.event_count(), 4);

        // Test Ctrl+C
        let event = source.next_event().unwrap();
        if let Event::Key(key_event) = event {
            assert_eq!(key_event.code, KeyCode::Char('c'));
            assert_eq!(key_event.modifiers, KeyModifiers::CONTROL);
        }

        // Test colon
        let event = source.next_event().unwrap();
        if let Event::Key(key_event) = event {
            assert_eq!(key_event.code, KeyCode::Char(':'));
        }
    }

    #[test]
    fn test_helper_fluent_interface() {
        let mut helper = ReplTestHelper::new();
        helper
            .type_text("GET /api/users")
            .press_enter()
            .enter_colon_command("send");

        assert_eq!(helper.event_source.event_count(), 19); // All the events added
    }
}
