//! # Terminal I/O Implementations
//!
//! Production implementations of I/O abstractions using crossterm.
//! All crossterm dependencies are isolated to this module.

use super::{EventStream, RenderStream};
use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::io::{self, Write};
use std::time::Duration;

/// Terminal-based event stream using crossterm
///
/// Reads events from the actual terminal using crossterm's event system.
/// This is the production implementation for real terminal interaction.
pub struct TerminalEventStream;

impl TerminalEventStream {
    /// Create a new terminal event stream
    pub fn new() -> Self {
        Self
    }
}

impl EventStream for TerminalEventStream {
    fn poll(&mut self, timeout: Duration) -> Result<bool> {
        event::poll(timeout).map_err(anyhow::Error::from)
    }

    fn read(&mut self) -> Result<Event> {
        event::read().map_err(anyhow::Error::from)
    }
}

/// Terminal-based render stream using crossterm
///
/// Renders to the actual terminal using crossterm's rendering system.
/// This is the production implementation for real terminal output.
pub struct TerminalRenderStream<W: Write> {
    writer: W,
}

impl TerminalRenderStream<io::Stdout> {
    /// Create a new terminal render stream using stdout
    pub fn new() -> Self {
        Self {
            writer: io::stdout(),
        }
    }
}

impl<W: Write> TerminalRenderStream<W> {
    /// Create a terminal render stream with custom writer
    pub fn with_writer(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> Write for TerminalRenderStream<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Convert bytes to string and use crossterm's Print to ensure proper execution
        if let Ok(text) = std::str::from_utf8(buf) {
            execute!(self.writer, crossterm::style::Print(text))?;
            Ok(buf.len())
        } else {
            // Fallback for non-UTF8 data
            self.writer.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Send> RenderStream for TerminalRenderStream<W> {
    fn clear_screen(&mut self) -> Result<()> {
        execute!(
            self.writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .map_err(anyhow::Error::from)
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        execute!(self.writer, cursor::MoveTo(x, y)).map_err(anyhow::Error::from)
    }

    fn hide_cursor(&mut self) -> Result<()> {
        execute!(self.writer, cursor::Hide).map_err(anyhow::Error::from)
    }

    fn show_cursor(&mut self) -> Result<()> {
        execute!(self.writer, cursor::Show).map_err(anyhow::Error::from)
    }

    fn get_size(&self) -> Result<super::TerminalSize> {
        terminal::size().map_err(anyhow::Error::from)
    }

    fn enter_alternate_screen(&mut self) -> Result<()> {
        execute!(self.writer, EnterAlternateScreen).map_err(anyhow::Error::from)
    }

    fn leave_alternate_screen(&mut self) -> Result<()> {
        execute!(self.writer, LeaveAlternateScreen).map_err(anyhow::Error::from)
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode().map_err(anyhow::Error::from)
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        terminal::disable_raw_mode().map_err(anyhow::Error::from)
    }
}

impl Default for TerminalEventStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TerminalRenderStream<io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}
