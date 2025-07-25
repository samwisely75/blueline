//! # Command Pattern Implementation
//!
//! This module defines the command pattern infrastructure for handling vim-style key events.
//! It includes both legacy Command trait and new CommandV2 trait with relevancy filtering.
//!
//! ## Design Principles
//!
//! - **Single Responsibility**: Each command implementation handles one specific action
//! - **Stateless**: Commands don't hold state, they operate on provided AppState
//! - **Pane Awareness**: Commands check current pane and act accordingly
//! - **Graceful Ignoring**: Commands ignore events they don't handle
//!
//! ## Example Usage
//!
//! ```rust
//! let command = MoveCursorLeftCommand;
//! let result = command.process(key_event, &mut app_state);
//! ```

use anyhow::Result;
use crossterm::event::KeyEvent;

use super::model::AppState;

/// Trait for processing user input events and updating application state.
///
/// Each vim command (movement, editing, mode changes) implements this trait.
/// Commands are stateless and operate on the provided AppState, following
/// the principle that the Controller orchestrates but the Commands execute.
///
/// ## Implementation Guidelines
///
/// 1. **Return early for irrelevant events**: Check if the command applies
///    to current mode/pane and return `Ok(false)` if not.
///
/// 2. **Update state atomically**: Make all related state changes together
///    to maintain consistency.
///
/// 3. **Respect pane boundaries**: Request pane commands shouldn't affect
///    Response pane state and vice versa.
///
/// 4. **Handle errors gracefully**: Return meaningful errors for invalid
///    operations rather than panicking.
pub trait Command {
    /// Process a key event and potentially modify the application state.
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event to process
    /// * `state` - Mutable reference to the application state
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Event was handled and state was modified
    /// * `Ok(false)` - Event was not relevant to this command
    /// * `Err(_)` - An error occurred during processing
    ///
    /// # Pane Awareness
    ///
    /// Commands should check `state.current_pane` and only operate on the
    /// appropriate buffer. For example, cursor movement in Response pane
    /// should not affect Request buffer state.
    ///
    /// # Mode Awareness
    ///
    /// Commands should check `state.mode` and only process events relevant
    /// to their mode. For example, insert commands should ignore events
    /// when in Normal mode.
    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool>;

    /// Get a human-readable name for this command (for debugging/logging).
    fn name(&self) -> &'static str;

    /// Check if this command is relevant for the current state.
    ///
    /// This is an optimization to avoid unnecessary processing. The default
    /// implementation returns true (always try to process).
    fn is_relevant(&self, state: &AppState) -> bool {
        let _ = state; // Suppress unused parameter warning
        true
    }
}

/// Result of command execution with additional metadata.
///
/// Provides more context about what the command did, which helps
/// the view layer decide what needs to be re-rendered.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command handled the event
    pub handled: bool,

    /// Whether the content of any buffer changed
    pub content_changed: bool,

    /// Whether the cursor position changed
    pub cursor_moved: bool,

    /// Whether the mode changed (affects status line)
    pub mode_changed: bool,

    /// Whether the current pane changed
    pub pane_changed: bool,

    /// Whether scrolling occurred (affects full pane render)
    pub scroll_occurred: bool,

    /// Optional status message to display
    pub status_message: Option<String>,
}

impl CommandResult {
    /// Create a result indicating the event was not handled
    pub fn not_handled() -> Self {
        Self {
            handled: false,
            content_changed: false,
            cursor_moved: false,
            mode_changed: false,
            pane_changed: false,
            scroll_occurred: false,
            status_message: None,
        }
    }

    /// Create a result indicating simple cursor movement
    pub fn cursor_moved() -> Self {
        Self {
            handled: true,
            content_changed: false,
            cursor_moved: true,
            mode_changed: false,
            pane_changed: false,
            scroll_occurred: false,
            status_message: None,
        }
    }

    /// Create a result indicating content was modified
    pub fn content_changed() -> Self {
        Self {
            handled: true,
            content_changed: true,
            cursor_moved: false,
            mode_changed: false,
            pane_changed: false,
            scroll_occurred: false,
            status_message: None,
        }
    }

    /// Create a result with scrolling
    pub fn with_scroll(mut self) -> Self {
        self.scroll_occurred = true;
        self
    }

    /// Create a result with mode change
    pub fn with_mode_change(mut self) -> Self {
        self.mode_changed = true;
        self
    }

    /// Create a result with pane change
    pub fn with_pane_change(mut self) -> Self {
        self.pane_changed = true;
        self
    }

    /// Add a status message to the result
    pub fn with_message(mut self, message: String) -> Self {
        self.status_message = Some(message);
        self
    }
}
