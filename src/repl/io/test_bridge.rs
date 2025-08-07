//! Test bridge for connecting test harness to application streams
//!
//! This module provides a bridge pattern that allows tests to control
//! application I/O while respecting Rust's ownership requirements.

use super::{EventStream, RenderStream, TerminalSize};
use anyhow::Result;
use crossterm::event::Event;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// Type alias for shared event receiver
type SharedEventReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Event>>>;

/// Type alias for shared byte receiver
type SharedByteReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>;

/// Type alias for shared byte buffer
type SharedByteBuffer = Arc<Mutex<Vec<u8>>>;

/// Bridge for sending events from tests to the application
pub struct BridgedEventStream {
    receiver: SharedEventReceiver,
}

impl BridgedEventStream {
    /// Create a new bridged event stream with its controller
    pub fn new() -> (Self, EventStreamController) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let stream = BridgedEventStream {
            receiver: Arc::new(Mutex::new(receiver)),
        };

        let controller = EventStreamController { sender };

        (stream, controller)
    }
}

impl EventStream for BridgedEventStream {
    fn poll(&mut self, _timeout: Duration) -> Result<bool> {
        // Check if there are events available
        // We use try_lock to avoid blocking
        if let Ok(receiver) = self.receiver.try_lock() {
            Ok(!receiver.is_empty())
        } else {
            // If we can't get the lock, assume no events
            Ok(false)
        }
    }

    fn read(&mut self) -> Result<Event> {
        // Block until we get an event
        let receiver = self.receiver.clone();

        // Use blocking_recv in a blocking context
        let handle = tokio::runtime::Handle::current();
        let event = handle.block_on(async {
            let mut rx = receiver.lock().await;
            rx.recv().await
        });

        event.ok_or_else(|| anyhow::anyhow!("Event stream closed"))
    }
}

/// Controller for sending events to a BridgedEventStream
#[derive(Clone)]
pub struct EventStreamController {
    sender: mpsc::UnboundedSender<Event>,
}

impl EventStreamController {
    /// Send an event to the stream
    pub fn send_event(&self, event: Event) -> Result<()> {
        self.sender
            .send(event)
            .map_err(|_| anyhow::anyhow!("Failed to send event"))
    }
}

/// Bridge for capturing output from the application
pub struct BridgedRenderStream {
    sender: mpsc::UnboundedSender<Vec<u8>>,
    terminal_size: TerminalSize,
}

impl BridgedRenderStream {
    /// Create a new bridged render stream with its monitor
    pub fn new(size: TerminalSize) -> (Self, RenderStreamMonitor) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let stream = BridgedRenderStream {
            sender: sender.clone(),
            terminal_size: size,
        };

        let monitor = RenderStreamMonitor {
            receiver: Arc::new(Mutex::new(receiver)),
            captured: Arc::new(Mutex::new(Vec::new())),
        };

        (stream, monitor)
    }
}

impl Write for BridgedRenderStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Send the bytes to the monitor
        self.sender
            .send(buf.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl RenderStream for BridgedRenderStream {
    fn clear_screen(&mut self) -> Result<()> {
        self.write_all(b"\x1b[2J\x1b[H")?;
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        let seq = format!("\x1b[{};{}H", y + 1, x + 1);
        self.write_all(seq.as_bytes())?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.write_all(b"\x1b[?25l")?;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.write_all(b"\x1b[?25h")?;
        Ok(())
    }

    fn get_size(&self) -> Result<TerminalSize> {
        Ok(self.terminal_size)
    }

    fn enter_alternate_screen(&mut self) -> Result<()> {
        self.write_all(b"\x1b[?1049h")?;
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> Result<()> {
        self.write_all(b"\x1b[?1049l")?;
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        // No-op for testing
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        // No-op for testing
        Ok(())
    }
}

/// Monitor for capturing output from a BridgedRenderStream
#[derive(Clone)]
pub struct RenderStreamMonitor {
    receiver: SharedByteReceiver,
    captured: SharedByteBuffer,
}

impl RenderStreamMonitor {
    /// Process all pending output and add to captured buffer
    pub async fn process_output(&self) {
        let mut receiver = self.receiver.lock().await;
        let mut captured = self.captured.lock().await;

        while let Ok(bytes) = receiver.try_recv() {
            captured.extend_from_slice(&bytes);
        }
    }

    /// Get all captured output
    pub async fn get_captured(&self) -> Vec<u8> {
        self.captured.lock().await.clone()
    }

    /// Clear captured output
    pub async fn clear(&self) {
        self.captured.lock().await.clear();
    }

    /// Get captured output as string
    pub async fn get_captured_string(&self) -> String {
        String::from_utf8_lossy(&self.get_captured().await).to_string()
    }
}
