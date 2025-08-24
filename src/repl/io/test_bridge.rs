//! Test bridge for connecting test harness to application streams
//!
//! This module provides a bridge pattern that allows tests to control
//! application I/O while respecting Rust's ownership requirements.

use super::{EventStream, RenderStream, TerminalSize};
use anyhow::Result;
use crossterm::event::Event;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// Type alias for shared event queue
type SharedEventQueue = Arc<StdMutex<VecDeque<Event>>>;

/// Type alias for wake sender
type WakeSender = Arc<StdMutex<Option<std::sync::mpsc::Sender<()>>>>;

/// Type alias for shared byte receiver
type SharedByteReceiver = Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>;

/// Type alias for shared byte buffer
type SharedByteBuffer = Arc<Mutex<Vec<u8>>>;

/// Bridge for sending events from tests to the application
///
/// Uses a simple VecDeque with std::sync::Mutex to avoid async complications
/// in the sync EventStream interface
pub struct BridgedEventStream {
    events: SharedEventQueue,
    #[allow(dead_code)]
    waker: WakeSender,
    wake_receiver: Option<std::sync::mpsc::Receiver<()>>,
}

impl BridgedEventStream {
    /// Create a new bridged event stream with its controller
    pub fn new() -> (Self, EventStreamController) {
        let events = Arc::new(StdMutex::new(VecDeque::new()));
        let (wake_sender, wake_receiver) = std::sync::mpsc::channel();
        let waker = Arc::new(StdMutex::new(Some(wake_sender)));

        let stream = BridgedEventStream {
            events: events.clone(),
            waker: waker.clone(),
            wake_receiver: Some(wake_receiver),
        };

        let controller = EventStreamController { events, waker };

        (stream, controller)
    }
}

impl EventStream for BridgedEventStream {
    fn poll(&mut self, timeout: Duration) -> Result<bool> {
        // Check if we have events
        if let Ok(queue) = self.events.lock() {
            if !queue.is_empty() {
                return Ok(true);
            }
        }

        // Wait for a wake signal with timeout
        if let Some(receiver) = &self.wake_receiver {
            match receiver.recv_timeout(timeout) {
                Ok(_) => {
                    // Got a wake signal, check for events again
                    if let Ok(queue) = self.events.lock() {
                        Ok(!queue.is_empty())
                    } else {
                        Ok(false)
                    }
                }
                Err(_) => Ok(false), // Timeout or disconnected
            }
        } else {
            Ok(false)
        }
    }

    fn read(&mut self) -> Result<Event> {
        // Try to get an event from the queue
        if let Ok(mut queue) = self.events.lock() {
            if let Some(event) = queue.pop_front() {
                return Ok(event);
            }
        }

        // No events available, block until we get one
        loop {
            // Wait for a wake signal
            if let Some(receiver) = &self.wake_receiver {
                // Block indefinitely waiting for an event
                let _ = receiver.recv();
            }

            // Check for events again
            if let Ok(mut queue) = self.events.lock() {
                if let Some(event) = queue.pop_front() {
                    return Ok(event);
                }
            }
        }
    }
}

/// Controller for sending events to a BridgedEventStream
#[derive(Clone)]
pub struct EventStreamController {
    events: SharedEventQueue,
    waker: WakeSender,
}

impl EventStreamController {
    /// Send an event to the stream
    pub fn send_event(&self, event: Event) -> Result<()> {
        // Add event to queue
        if let Ok(mut queue) = self.events.lock() {
            queue.push_back(event);
        } else {
            return Err(anyhow::anyhow!("Failed to lock event queue"));
        }

        // Send wake signal
        if let Ok(waker_guard) = self.waker.lock() {
            if let Some(sender) = &*waker_guard {
                let _ = sender.send(()); // Ignore error if receiver is gone
            }
        }

        Ok(())
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

    /// Inject data directly into captured buffer (for test simulation)
    pub async fn inject_data(&self, data: &[u8]) {
        let mut captured = self.captured.lock().await;
        captured.extend_from_slice(data);
    }
}
