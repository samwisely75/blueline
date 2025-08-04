//! Test Event Source Implementation
//!
//! Test implementation of EventSource that uses a queue of pre-programmed
//! events, allowing deterministic testing without requiring a real terminal.

use super::event_source::{EventSource, TestEventSource as TestEventSourceTrait};
use anyhow::Result;
use crossterm::event::{Event, KeyEvent};
use std::collections::VecDeque;
use std::time::Duration;

/// Test event source that provides events from a pre-programmed queue
#[derive(Debug, Clone)]
pub struct TestEventSource {
    events: VecDeque<Event>,
    always_ready: bool,
}

impl TestEventSource {
    /// Create a new test event source with an empty event queue
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            always_ready: true,
        }
    }

    /// Create a test event source with pre-populated events
    pub fn with_events(events: Vec<Event>) -> Self {
        Self {
            events: events.into(),
            always_ready: true,
        }
    }

    /// Set whether poll() should always return true (for testing scenarios)
    ///
    /// When true, poll() returns true if there are events queued.
    /// When false, poll() always returns false (useful for timeout testing).
    pub fn set_always_ready(&mut self, ready: bool) {
        self.always_ready = ready;
    }
}

impl Default for TestEventSource {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSource for TestEventSource {
    fn poll(&mut self, _timeout: Duration) -> Result<bool> {
        if !self.always_ready {
            return Ok(false);
        }

        // Return true if we have events available
        Ok(!self.events.is_empty())
    }

    fn read(&mut self) -> Result<Event> {
        self.events
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No events available in test queue"))
    }

    fn is_exhausted(&self) -> bool {
        self.events.is_empty()
    }
}

impl TestEventSourceTrait for TestEventSource {
    fn push_key_event(&mut self, key_event: KeyEvent) {
        self.events.push_back(Event::Key(key_event));
    }

    fn push_resize_event(&mut self, width: u16, height: u16) {
        self.events.push_back(Event::Resize(width, height));
    }

    fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    fn clear_events(&mut self) {
        self.events.clear();
    }

    fn pending_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_event_source_starts_empty() {
        let source = TestEventSource::new();
        assert!(source.is_exhausted());
        assert_eq!(source.pending_count(), 0);
    }

    #[test]
    fn test_push_and_read_key_event() -> Result<()> {
        let mut source = TestEventSource::new();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        source.push_key_event(key_event);

        assert!(!source.is_exhausted());
        assert_eq!(source.pending_count(), 1);

        // Should have events available
        assert!(source.poll(Duration::from_millis(1))?);

        // Read the event
        let event = source.read()?;
        match event {
            Event::Key(k) => {
                assert_eq!(k.code, KeyCode::Char('a'));
                assert_eq!(k.modifiers, KeyModifiers::NONE);
            }
            _ => panic!("Expected key event"),
        }

        assert!(source.is_exhausted());
        Ok(())
    }

    #[test]
    fn test_push_resize_event() -> Result<()> {
        let mut source = TestEventSource::new();

        source.push_resize_event(80, 24);

        assert!(source.poll(Duration::from_millis(1))?);

        let event = source.read()?;
        match event {
            Event::Resize(w, h) => {
                assert_eq!(w, 80);
                assert_eq!(h, 24);
            }
            _ => panic!("Expected resize event"),
        }

        Ok(())
    }

    #[test]
    fn test_clear_events() {
        let mut source = TestEventSource::new();

        source.push_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        source.push_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));

        assert_eq!(source.pending_count(), 2);

        source.clear_events();

        assert_eq!(source.pending_count(), 0);
        assert!(source.is_exhausted());
    }

    #[test]
    fn test_poll_returns_false_when_not_ready() -> Result<()> {
        let mut source = TestEventSource::new();
        source.push_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        // Set to not ready
        source.set_always_ready(false);

        // Should return false even though we have events
        assert!(!source.poll(Duration::from_millis(1))?);

        // Set back to ready
        source.set_always_ready(true);

        // Now should return true
        assert!(source.poll(Duration::from_millis(1))?);

        Ok(())
    }

    #[test]
    fn test_read_empty_queue_returns_error() {
        let mut source = TestEventSource::new();

        let result = source.read();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No events available"));
    }

    #[test]
    fn test_with_events_constructor() -> Result<()> {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
            Event::Resize(80, 24),
        ];

        let mut source = TestEventSource::with_events(events);

        assert_eq!(source.pending_count(), 2);

        // Read first event
        assert!(source.poll(Duration::from_millis(1))?);
        let event1 = source.read()?;
        assert!(matches!(event1, Event::Key(_)));

        // Read second event
        assert!(source.poll(Duration::from_millis(1))?);
        let event2 = source.read()?;
        assert!(matches!(event2, Event::Resize(80, 24)));

        // Should be exhausted now
        assert!(source.is_exhausted());
        assert!(!source.poll(Duration::from_millis(1))?);

        Ok(())
    }
}
