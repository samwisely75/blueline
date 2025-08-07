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
        io::{MockEventStream, VteRenderStream},
    },
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use cucumber::World;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, trace, warn};

use super::terminal_state::TerminalState;

/// The Cucumber World for Blueline integration tests
///
/// This struct maintains all test state and provides methods for
/// interacting with the application under test.
#[derive(World)]
pub struct BluelineWorld {
    /// The application controller instance (wrapped for async safety)
    app: Arc<Mutex<Option<AppController<MockEventStream, VteRenderStream>>>>,

    /// Mock event stream for providing input
    event_stream: Arc<Mutex<MockEventStream>>,

    /// VTE render stream for capturing output
    render_stream: Arc<Mutex<VteRenderStream>>,

    /// Terminal dimensions for testing
    terminal_size: (u16, u16),

    /// Last parsed terminal state (for assertions)
    last_terminal_state: Option<TerminalState>,

    /// Test profile path (temporary)
    profile_path: Option<String>,
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
            app: Arc::new(Mutex::new(None)),
            event_stream: Arc::new(Mutex::new(MockEventStream::empty())),
            render_stream: Arc::new(Mutex::new(VteRenderStream::with_size((80, 24)))),
            terminal_size: (80, 24),
            last_terminal_state: None,
            profile_path: None,
        }
    }
}

impl BluelineWorld {
    /// Initialize the world for a new scenario
    pub async fn initialize(&mut self) {
        debug!("Initializing BluelineWorld for new scenario");

        // Clear any previous state
        self.cleanup().await;

        // Create fresh streams
        self.event_stream = Arc::new(Mutex::new(MockEventStream::empty()));
        self.render_stream = Arc::new(Mutex::new(VteRenderStream::with_size(self.terminal_size)));
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
        if let Some(_app) = self.app.lock().await.take() {
            debug!("Shutting down application");
            // App will be dropped automatically
            // Note: In a real implementation, we'd send a quit event and wait for shutdown
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

        // Parse command line arguments
        // CommandLineArgs::parse_from expects the program name as first arg
        let mut full_args = vec!["blueline".to_string()];
        full_args.extend(args);

        let cmd_args = CommandLineArgs::parse_from(full_args);

        // Create app controller with mock streams
        let event_stream = MockEventStream::empty();
        let render_stream = VteRenderStream::with_size(self.terminal_size);

        debug!("Creating AppController with mock streams");
        let app = AppController::with_io_streams(cmd_args, event_stream, render_stream)?;

        // Store the app
        *self.app.lock().await = Some(app);

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
        let event = Event::Key(KeyEvent::new(code, modifiers));
        self.event_stream.lock().await.push_event(event);
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
    pub async fn tick(&mut self) -> Result<()> {
        if let Some(ref mut _app) = *self.app.lock().await {
            // In a real implementation, we'd call app.tick() or similar
            // For now, this is a placeholder
            Ok(())
        } else {
            Err(anyhow::anyhow!("Application not started"))
        }
    }

    /// Get the current terminal state
    pub async fn get_terminal_state(&mut self) -> TerminalState {
        trace!("Getting current terminal state");
        let render_stream = self.render_stream.lock().await;
        let state = TerminalState::from_render_stream(&render_stream);
        self.last_terminal_state = Some(state.clone());
        trace!("Terminal state captured");
        state
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
