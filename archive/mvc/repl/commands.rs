//! # Command Pattern Implementation and Command Modules
//!
//! This module defines the command pattern infrastructure for handling vim-style key events
//! and organizes all command implementations.
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
//! ```rust,no_run
//! use blueline::repl::commands::movement::MoveCursorLeftCommand;
//! use blueline::repl::commands::Command;
//! use blueline::repl::model::AppState;
//! use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
//!
//! let command = MoveCursorLeftCommand;
//! let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
//! let mut app_state = AppState::new((80, 24), false);
//! let result = command.process(key_event, &mut app_state);
//! ```

use anyhow::Result;
use crossterm::event::KeyEvent;

use super::model::AppState;
use super::view_model::ViewModel;

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

    /// Check if this command is relevant for the current state and event.
    ///
    /// This allows commands to filter events before processing, improving
    /// performance and ensuring proper command precedence.
    ///
    /// # Arguments
    ///
    /// * `state` - Reference to the application state for checking mode/pane
    /// * `event` - The keyboard event to check
    ///
    /// # Returns
    ///
    /// * `true` - Command should attempt to process this event
    /// * `false` - Command should ignore this event
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool;

    /// Get a human-readable name for this command (for debugging/logging).
    fn name(&self) -> &'static str;
}

/// MVVM Command trait for processing user input events through ViewModel.
///
/// This is the new MVVM approach where commands delegate to ViewModel methods
/// instead of directly manipulating AppState. The ViewModel handles all the
/// business logic, event emission, and display concerns.
///
/// ## Benefits over Legacy Command Trait
///
/// 1. **Separation of Concerns**: Commands focus on input mapping, ViewModel handles logic
/// 2. **Event-Driven**: Automatic event emission for model changes
/// 3. **Centralized Logic**: All cursor movement, scrolling logic in ViewModel
/// 4. **Better Testing**: Can test ViewModel methods independently
/// 5. **Consistency**: Same logic path regardless of input source
///
/// ## Implementation Guidelines
///
/// 1. **Delegate to ViewModel**: Commands should primarily call ViewModel methods
/// 2. **Check Relevancy**: Use `is_relevant()` to filter applicable events
/// 3. **Handle Errors**: Return meaningful errors from ViewModel operations
/// 4. **Keep Simple**: Commands should be thin wrappers around ViewModel calls
pub trait MvvmCommand {
    /// Check if this command is relevant for the current state and event.
    ///
    /// # Arguments
    ///
    /// * `view_model` - Reference to the view model for state checking
    /// * `event` - The keyboard event to check
    ///
    /// # Returns
    ///
    /// * `true` - Command can handle this event in current state
    /// * `false` - Command is not relevant for this event/state
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool;
    
    /// Process a key event through the ViewModel.
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event to process
    /// * `view_model` - Mutable reference to the view model
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Event was handled successfully
    /// * `Ok(false)` - Event was not relevant (should not happen if is_relevant works)
    /// * `Err(_)` - An error occurred during processing
    ///
    /// # Event Flow
    ///
    /// 1. Command delegates to appropriate ViewModel method
    /// 2. ViewModel updates models and emits events
    /// 3. View subscribers receive events and update display
    /// 4. Controller receives success/failure result
    fn execute(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool>;
    
    /// Get a human-readable name for this command (for debugging/logging).
    fn name(&self) -> &'static str;
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

// Future display-space architecture (commented out for now)
// /// Display state for a single pane (Request or Response)
// #[derive(Debug, Clone, PartialEq)]
// pub struct PaneDisplayState {
//     /// Display line offset (top visible display line)
//     pub display_scroll_offset: usize,
//     /// Logical line offset (primarily for line numbers)
//     pub logical_scroll_offset: usize,
//     /// Cursor position in display coordinates
//     pub display_cursor: (usize, usize), // (display_line, display_col)
//     /// Cursor position in logical coordinates (derived from display)
//     pub logical_cursor: (usize, usize), // (logical_line, logical_col)
// }

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

// Command modules
pub mod command_line;
pub mod editing;
pub mod mode;
pub mod movement;
pub mod window;

// Re-export commonly used commands
pub use command_line::*;
pub use editing::*;
pub use mode::*;
pub use movement::*;
pub use window::*;
