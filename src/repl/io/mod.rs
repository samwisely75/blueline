//! # I/O Abstraction Layer
//!
//! Provides clean trait abstractions for input/output streams to enable
//! dependency injection without polluting production code.
//!
//! ## Design Principles
//!
//! - **EventStream**: Abstracts input events (keyboard, terminal resize, etc.)
//! - **RenderStream**: Abstracts output rendering (cursor, colors, screen manipulation)
//! - **Clean Separation**: All terminal-specific code isolated to implementations
//! - **Dependency Injection**: Enables testing without terminal dependencies
//!
//! ## Architecture
//!
//! ```text
//! Production:  AppController ──▶ TerminalEventStream ──▶ crossterm::event::read()
//!                            ──▶ TerminalRenderStream ──▶ crossterm::execute!()
//!
//! Testing:     AppController ──▶ MockEventStream     ──▶ VecDeque<Event>
//!                            ──▶ MockRenderStream    ──▶ Vec<RenderCommand>
//! ```

use anyhow::Result;
use crossterm::event::Event;
use std::io::Write;
use std::time::Duration;

pub mod mock;
pub mod terminal;

// Re-export terminal implementations for convenience
pub use terminal::{TerminalEventStream, TerminalRenderStream};

// Re-export mock implementations for testing
pub use mock::{MockEventStream, MockRenderStream, TerminalStateInfo, VteRenderStream};

/// Type alias for terminal size (width, height)
pub type TerminalSize = (u16, u16);

/// Input event stream abstraction
///
/// Abstracts the source of input events to enable clean dependency injection.
/// Production implementations use crossterm for real terminal input.
/// Test implementations can provide pre-programmed event sequences.
pub trait EventStream: Send {
    /// Check if events are available without blocking
    ///
    /// Returns true if events are ready to be read within the timeout period.
    /// This is equivalent to crossterm::event::poll().
    fn poll(&mut self, timeout: Duration) -> Result<bool>;

    /// Read the next available event
    ///
    /// This should only be called after poll() returns true.
    /// Returns the next event from the input source.
    fn read(&mut self) -> Result<Event>;
}

/// Output render stream abstraction  
///
/// Abstracts terminal rendering operations to enable clean dependency injection.
/// Production implementations use crossterm for real terminal output.
/// Test implementations can capture and verify render commands.
pub trait RenderStream: Write + Send {
    /// Clear the entire screen
    fn clear_screen(&mut self) -> Result<()>;

    /// Move cursor to specific position (column, row)
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()>;

    /// Hide the cursor
    fn hide_cursor(&mut self) -> Result<()>;

    /// Show the cursor
    fn show_cursor(&mut self) -> Result<()>;

    /// Get terminal size as (width, height)
    fn get_size(&self) -> Result<TerminalSize>;

    /// Enter alternate screen buffer
    fn enter_alternate_screen(&mut self) -> Result<()>;

    /// Leave alternate screen buffer  
    fn leave_alternate_screen(&mut self) -> Result<()>;

    /// Enable terminal raw mode
    fn enable_raw_mode(&mut self) -> Result<()>;

    /// Disable terminal raw mode
    fn disable_raw_mode(&mut self) -> Result<()>;
}
