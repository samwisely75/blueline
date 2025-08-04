//! # Event Source Abstraction - Core TTY Solution
//!
//! This module provides the **EventSource trait** - the key innovation that enables
//! headless testing of terminal applications without TTY access.
//!
//! ## Problem Solved
//!
//! Terminal applications traditionally require `crossterm::event::read()` which:
//! - **Blocks indefinitely** waiting for keyboard input
//! - **Requires a real TTY** (terminal device)  
//! - **Cannot run in CI** environments (no interactive terminal)
//! - **Cannot be easily mocked** due to crossterm's design
//!
//! ## Solution Architecture
//!
//! The EventSource trait abstracts the event input mechanism:
//!
//! ```text
//! Production:   AppController ──▶ TerminalEventSource ──▶ crossterm::event::read()
//! Testing:      AppController ──▶ TestEventSource     ──▶ VecDeque<Event>
//! ```
//!
//! ## Key Benefits
//!
//! 1. **CI Compatible**: Tests run without TTY requirements
//! 2. **Deterministic**: Test events are pre-programmed and repeatable  
//! 3. **Real Behavior**: Production uses actual crossterm, maintaining fidelity
//! 4. **Zero Overhead**: Trait is zero-cost abstraction in production
//! 5. **Drop-in Replacement**: No changes needed to core application logic
//!
//! ## Usage Pattern
//!
//! ```rust,no_run
//! use blueline::{AppController, TestEventSource};
//! use blueline::cmd_args::CommandLineArgs;
//! use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};
//!
//! let cmd_args = CommandLineArgs::parse_from(["test"]);
//!
//! // Production
//! let app_controller = AppController::new(cmd_args.clone()); // Uses TerminalEventSource
//!
//! // Testing  
//! let test_events = TestEventSource::with_events(vec![
//!     Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE)),
//!     Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
//! ]);
//! let app_controller = AppController::with_event_source(cmd_args, test_events);
//! ```
//!
//! This abstraction enables comprehensive integration testing while maintaining
//! production behavior and performance.

use anyhow::Result;
use crossterm::event::{Event, KeyEvent};
use std::time::Duration;

/// Trait for abstracting event input sources
///
/// This allows us to inject different event sources for production vs testing:
/// - Production: Uses crossterm to read from terminal
/// - Testing: Uses a queue of pre-programmed events
pub trait EventSource {
    /// Check if events are available without blocking
    ///
    /// Returns true if events are ready to be read, false if timeout elapsed.
    /// This is equivalent to crossterm::event::poll()
    fn poll(&mut self, timeout: Duration) -> Result<bool>;

    /// Read the next available event
    ///
    /// This should only be called after poll() returns true.
    /// Returns the next event from the input source.
    fn read(&mut self) -> Result<Event>;

    /// Check if the event source is exhausted (for testing)
    ///
    /// Returns true if no more events are available and none will be added.
    /// For terminal sources, this should always return false.
    /// For test sources, this indicates all queued events have been consumed.
    fn is_exhausted(&self) -> bool {
        false
    }
}

/// Helper trait for test event sources to inject events
pub trait TestEventSource: EventSource {
    /// Add a key event to the test queue
    fn push_key_event(&mut self, key_event: KeyEvent);

    /// Add a resize event to the test queue
    fn push_resize_event(&mut self, width: u16, height: u16);

    /// Add a generic event to the test queue
    fn push_event(&mut self, event: Event);

    /// Clear all pending events from the queue
    fn clear_events(&mut self);

    /// Get the number of pending events in the queue
    fn pending_count(&self) -> usize;
}
