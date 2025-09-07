//! Terminal Event Source Implementation
//!
//! Production implementation of EventSource that uses crossterm to read
//! real terminal events from stdin.

use super::event_source::EventSource;
use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

/// Production event source that reads from terminal via crossterm
#[derive(Debug, Default)]
pub struct TerminalEventSource;

impl TerminalEventSource {
    /// Create a new terminal event source
    pub fn new() -> Self {
        Self
    }
}

impl EventSource for TerminalEventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool> {
        event::poll(timeout).map_err(anyhow::Error::from)
    }

    fn read(&mut self) -> Result<Event> {
        event::read().map_err(anyhow::Error::from)
    }

    fn is_exhausted(&self) -> bool {
        // Terminal event source is never exhausted - it can always potentially receive more events
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_event_source_never_exhausted() {
        let source = TerminalEventSource::new();
        assert!(!source.is_exhausted());
    }

    #[test]
    fn terminal_event_source_can_be_created() {
        let _source = TerminalEventSource::new();
        // Just verify it compiles and constructs
    }
}
