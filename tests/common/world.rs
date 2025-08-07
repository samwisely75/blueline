//! Cucumber World implementation for Blueline integration tests
//!
//! This module provides the World struct that maintains test state across
//! Cucumber steps. It follows clean architecture principles with no global state.

#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::arc_with_non_send_sync)]

use anyhow::Result;
use blueline::{
    cmd_args::CommandLineArgs,
    repl::{
        controllers::app_controller::AppController,
        io::{
            test_bridge::{
                BridgedEventStream, BridgedRenderStream, EventStreamController, RenderStreamMonitor,
            },
            VteRenderStream,
        },
    },
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use cucumber::World;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace, warn};

use super::terminal_state::TerminalState;

/// The Cucumber World for Blueline integration tests
///
/// This struct maintains all test state and provides methods for
/// interacting with the application under test.
#[derive(World)]
pub struct BluelineWorld {
    /// Task handle for the running application
    app_task: Option<JoinHandle<Result<()>>>,

    /// Controller for sending events to the app
    event_controller: Option<EventStreamController>,

    /// Monitor for capturing output from the app
    render_monitor: Option<RenderStreamMonitor>,

    /// VTE parser for interpreting terminal output
    vte_parser: Arc<Mutex<VteRenderStream>>,

    /// Channel to signal app shutdown
    shutdown_tx: Option<mpsc::Sender<()>>,

    /// Terminal dimensions for testing
    terminal_size: (u16, u16),

    /// Last parsed terminal state (for assertions)
    last_terminal_state: Option<TerminalState>,

    /// Test profile path (temporary)
    profile_path: Option<String>,

    /// Whether the app is currently running
    app_running: bool,
}

impl std::fmt::Debug for BluelineWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluelineWorld")
            .field("terminal_size", &self.terminal_size)
            .field("has_app", &"<AppController>")
            .field("has_profile_path", &self.profile_path.is_some())
            .finish()
    }
}

impl Default for BluelineWorld {
    fn default() -> Self {
        Self {
            app_task: None,
            event_controller: None,
            render_monitor: None,
            vte_parser: Arc::new(Mutex::new(VteRenderStream::with_size((80, 24)))),
            shutdown_tx: None,
            terminal_size: (80, 24),
            last_terminal_state: None,
            profile_path: None,
            app_running: false,
        }
    }
}

impl BluelineWorld {
    /// Initialize the world for a new scenario
    pub async fn initialize(&mut self) {
        debug!("Initializing BluelineWorld for new scenario");

        // Clear any previous state
        self.cleanup().await;

        // Reset VTE parser
        self.vte_parser = Arc::new(Mutex::new(VteRenderStream::with_size(self.terminal_size)));
        self.last_terminal_state = None;

        trace!(
            "World initialized with terminal size {:?}",
            self.terminal_size
        );
    }

    /// Clean up after a scenario
    pub async fn cleanup(&mut self) {
        debug!("Cleaning up BluelineWorld");

        // Shutdown the app if running
        if self.app_running {
            debug!("Shutting down application");

            // Send quit event to gracefully shutdown
            self.send_key_event(KeyCode::Char('q'), KeyModifiers::CONTROL)
                .await;

            // Give app time to process quit
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Signal shutdown if channel exists
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(()).await;
            }

            // Wait for app task to complete
            if let Some(task) = self.app_task.take() {
                match tokio::time::timeout(Duration::from_secs(2), task).await {
                    Ok(Ok(Ok(()))) => debug!("App shut down cleanly"),
                    Ok(Ok(Err(e))) => warn!("App returned error: {}", e),
                    Ok(Err(e)) => warn!("App task panicked: {:?}", e),
                    Err(_) => {
                        warn!("App shutdown timed out");
                    }
                }
            }

            self.app_running = false;
        }

        // Clear terminal state
        self.last_terminal_state = None;

        // Clean up temporary profile if created
        if let Some(path) = &self.profile_path {
            debug!("Removing temporary profile at: {}", path);
            if let Err(e) = std::fs::remove_file(path) {
                warn!("Failed to remove temporary profile: {}", e);
            }
            self.profile_path = None;
        }
    }

    /// Start the application with given arguments
    pub async fn start_app(&mut self, args: Vec<String>) -> Result<()> {
        info!("Starting application with args: {:?}", args);

        // Ensure any previous app is cleaned up
        if self.app_running {
            self.cleanup().await;
        }

        // Parse command line arguments
        // CommandLineArgs::parse_from expects the program name as first arg
        let mut full_args = vec!["blueline".to_string()];
        full_args.extend(args);

        let cmd_args = CommandLineArgs::parse_from(full_args);

        // Create the bridge components
        let (event_stream, event_controller) = BridgedEventStream::new();
        let (render_stream, render_monitor) = BridgedRenderStream::new(self.terminal_size);

        // Store the controllers for test access
        self.event_controller = Some(event_controller);
        self.render_monitor = Some(render_monitor);

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        debug!("Creating AppController with bridged streams");

        // Spawn the app in a background task (now works with Send traits!)
        let app_task = tokio::spawn(async move {
            // Create the AppController with the bridged streams
            let mut app = AppController::with_io_streams(cmd_args, event_stream, render_stream)?;

            // Run the app with shutdown support
            tokio::select! {
                result = app.run() => {
                    debug!("App run completed: {:?}", result);
                    result
                }
                _ = shutdown_rx.recv() => {
                    debug!("App received shutdown signal");
                    Ok(())
                }
            }
        });

        self.app_task = Some(app_task);
        self.app_running = true;

        // Give the app a moment to initialize
        tokio::time::sleep(Duration::from_millis(200)).await;

        info!("Application started successfully");
        Ok(())
    }

    /// Send a key event to the application
    pub async fn send_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        trace!(
            "Sending key event: {:?} with modifiers: {:?}",
            code,
            modifiers
        );

        if let Some(controller) = &self.event_controller {
            let event = Event::Key(KeyEvent::new(code, modifiers));
            if let Err(e) = controller.send_event(event) {
                error!("Failed to send key event: {}", e);
            }
        } else {
            warn!("Cannot send key event: app not started");
        }
    }

    /// Send a string of characters as key events
    pub async fn type_text(&mut self, text: &str) {
        debug!("Typing text: '{}'", text);
        for ch in text.chars() {
            self.send_key_event(KeyCode::Char(ch), KeyModifiers::empty())
                .await;
        }
    }

    /// Send an Enter key press
    pub async fn press_enter(&mut self) {
        self.send_key_event(KeyCode::Enter, KeyModifiers::empty())
            .await;
    }

    /// Send an Escape key press
    pub async fn press_escape(&mut self) {
        self.send_key_event(KeyCode::Esc, KeyModifiers::empty())
            .await;
    }

    /// Process events (tick the application)
    /// This allows time for the app to process events and produce output
    pub async fn tick(&mut self) -> Result<()> {
        if self.app_running {
            // Give the app time to process events
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Process any pending output from the render stream
            if let Some(monitor) = &self.render_monitor {
                monitor.process_output().await;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Application not started"))
        }
    }

    /// Get the current terminal state
    pub async fn get_terminal_state(&mut self) -> TerminalState {
        trace!("Getting current terminal state");

        if let Some(monitor) = &self.render_monitor {
            // Process any pending output
            monitor.process_output().await;

            // Get the captured output and feed it to our VTE parser
            let output = monitor.get_captured().await;

            // Create a VTE stream and write the output to it for parsing
            let mut vte_parser = self.vte_parser.lock().await;
            vte_parser.clear_captured(); // Clear previous data
            let _ = vte_parser.write(&output); // Write captured output to VTE parser

            // Create terminal state from the parsed output
            let state = TerminalState::from_render_stream(&vte_parser);
            self.last_terminal_state = Some(state.clone());
            trace!("Terminal state captured from {} bytes", output.len());
            state
        } else {
            warn!("Cannot get terminal state: app not started");
            TerminalState::default()
        }
    }

    /// Check if terminal contains text
    pub async fn terminal_contains(&mut self, text: &str) -> bool {
        debug!("Checking if terminal contains: '{}'", text);
        let state = self.get_terminal_state().await;
        let contains = state.contains(text);
        trace!("Terminal contains '{}': {}", text, contains);
        contains
    }

    /// Get a specific line from the terminal
    pub async fn get_terminal_line(&mut self, line_num: usize) -> Option<String> {
        let state = self.get_terminal_state().await;
        state.get_line(line_num)
    }

    /// Assert cursor is at a specific position
    pub async fn assert_cursor_at(&mut self, col: u16, row: u16) {
        let state = self.get_terminal_state().await;
        state.assert_cursor_at(col, row);
    }

    /// Set terminal size for testing
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
        // If app is running, we'd need to send a resize event
    }

    /// Create a temporary test profile
    pub async fn create_test_profile(&mut self, content: &str) -> Result<()> {
        debug!("Creating temporary test profile");
        use std::io::Write;
        let temp_file = tempfile::NamedTempFile::new()?;
        let path = temp_file.path().to_string_lossy().to_string();

        let mut file = std::fs::File::create(&path)?;
        file.write_all(content.as_bytes())?;

        info!("Created test profile at: {}", path);
        self.profile_path = Some(path.clone());
        Ok(())
    }

    /// Debug helper: log current terminal state
    pub async fn debug_terminal(&mut self) {
        debug!("Dumping current terminal state");
        let state = self.get_terminal_state().await;
        state.debug_print();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_world_initialization() {
        let mut world = BluelineWorld::default();
        world.initialize().await;

        // Verify initial state
        assert!(world.last_terminal_state.is_none());
        assert_eq!(world.terminal_size, (80, 24));
    }

    #[tokio::test]
    async fn test_send_key_events() {
        let mut world = BluelineWorld::default();
        world.initialize().await;

        // Send some key events
        world.type_text("hello").await;
        world.press_enter().await;

        // Verify events were queued (would need app running to process them)
        // This is mainly testing that the methods don't panic
    }
}
