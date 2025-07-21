//! # Controller - Main REPL Controller and Command Coordination
//!
//! This module contains the main controller that coordinates the MVC components.
//! It manages the event loop, maintains the command registry, and orchestrates
//! interactions between models, views, and commands.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────┐

#![allow(dead_code)] // Allow unused code during refactoring
#![allow(clippy::type_complexity)] // Allow complex types during refactoring
//! │   ReplController    │
//! │                     │
//! │  ┌───────────────┐  │    ┌─────────────┐
//! │  │   Commands    │  │────▶│ AppState    │
//! │  │   Registry    │  │    │ (Model)     │
//! │  └───────────────┘  │    └─────────────┘
//! │           │         │           │
//! │           ▼         │           ▼
//! │  ┌───────────────┐  │    ┌─────────────┐
//! │  │  Event Loop   │  │    │ ViewManager │
//! │  └───────────────┘  │    │ (View)      │
//! └─────────────────────┘    └─────────────┘
//! ```

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use bluenote::{HttpClient, IniProfile};

use super::{
    command::{Command, CommandResult},
    commands::{
        DeleteCharCommand, EnterCommandModeCommand, EnterInsertModeCommand, ExitInsertModeCommand,
        InsertCharCommand, InsertNewLineCommand, MoveCursorDownCommand, MoveCursorLeftCommand,
        MoveCursorLineEndCommand, MoveCursorLineStartCommand, MoveCursorRightCommand,
        MoveCursorUpCommand, SwitchPaneCommand,
    },
    model::AppState,
    view::{create_default_view_manager, ViewManager},
};

/// Main controller that orchestrates the REPL application.
///
/// This is the central coordinator that:
/// - Manages the event loop
/// - Maintains command registry  
/// - Coordinates model updates and view rendering
/// - Handles application lifecycle
pub struct ReplController {
    state: AppState,
    view_manager: ViewManager,
    commands: Vec<Box<dyn Command>>,
    client: HttpClient,
    profile: IniProfile,
}

impl ReplController {
    /// Create a new REPL controller
    pub fn new(profile: IniProfile, verbose: bool) -> Result<Self> {
        let client = HttpClient::new(&profile)?;
        let terminal_size = terminal::size()?;

        let state = AppState::new(terminal_size, verbose);
        let view_manager = create_default_view_manager();

        let mut controller = Self {
            state,
            view_manager,
            commands: Vec::new(),
            client,
            profile,
        };

        // Register default commands
        controller.register_default_commands();

        Ok(controller)
    }

    /// Run the REPL event loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        self.view_manager.initialize_terminal()?;

        // Initial render
        self.view_manager.render_full(&self.state)?;

        let result = self.event_loop().await;

        // Cleanup
        self.view_manager.cleanup_terminal()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        result
    }

    /// Main event processing loop
    async fn event_loop(&mut self) -> Result<()> {
        loop {
            match event::read()? {
                Event::Key(key) => {
                    let should_quit = self.handle_key_event(key).await?;
                    if should_quit {
                        break;
                    }
                }
                Event::Resize(width, height) => {
                    self.state.update_terminal_size((width, height));
                    self.view_manager.render_full(&self.state)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle a key event by dispatching to appropriate commands
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        let mut should_quit = false;
        let mut any_handled = false;
        let mut command_results = Vec::new();

        // Track old state for change detection
        let old_mode = self.state.mode.clone();
        let old_pane = self.state.current_pane;
        let old_request_scroll = self.state.request_buffer.scroll_offset;
        let old_response_scroll = self.state.response_buffer.as_ref().map(|r| r.scroll_offset);
        let old_request_pane_height = self.state.request_pane_height;

        // Try each command until one handles the event
        for command in &self.commands {
            // Use the unified Command trait (CommandV2 is auto-implemented via blanket impl)
            if !command.is_relevant(&self.state) {
                continue;
            }

            let handled = command.process(key, &mut self.state)?;
            if handled {
                // Create a basic result for all commands
                command_results.push(CommandResult {
                    handled: true,
                    content_changed: false, // Conservative assumption
                    cursor_moved: true,     // Conservative assumption
                    mode_changed: false,
                    pane_changed: false,
                    scroll_occurred: false,
                    status_message: None,
                });
                any_handled = true;
                break; // First handler wins
            }
        }

        // Handle special quit commands
        if matches!(key.code, crossterm::event::KeyCode::Char('c'))
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            should_quit = true;
        }

        // Determine what type of rendering is needed
        if any_handled {
            self.update_view_based_on_changes(
                &command_results,
                old_mode,
                old_pane,
                old_request_scroll,
                old_response_scroll,
                old_request_pane_height,
            )?;
        }

        Ok(should_quit)
    }

    /// Update the view based on detected changes
    fn update_view_based_on_changes(
        &mut self,
        results: &[CommandResult],
        old_mode: super::model::EditorMode,
        old_pane: super::model::Pane,
        old_request_scroll: usize,
        old_response_scroll: Option<usize>,
        old_request_pane_height: usize,
    ) -> Result<()> {
        // Check if scrolling occurred
        let scroll_occurred = self.state.request_buffer.scroll_offset != old_request_scroll
            || self.state.response_buffer.as_ref().map(|r| r.scroll_offset) != old_response_scroll;

        // Check if pane layout changed
        let pane_layout_changed = self.state.request_pane_height != old_request_pane_height;

        // Aggregate results to determine render strategy
        let any_mode_changed =
            results.iter().any(|r| r.mode_changed) || self.state.mode != old_mode;
        let any_pane_changed =
            results.iter().any(|r| r.pane_changed) || self.state.current_pane != old_pane;
        let any_scroll = results.iter().any(|r| r.scroll_occurred) || scroll_occurred;
        let any_content_changed = results.iter().any(|r| r.content_changed);
        let any_cursor_moved = results.iter().any(|r| r.cursor_moved);

        // Apply rendering strategy based on the same logic as the original
        let needs_full_render = any_mode_changed
            || any_pane_changed
            || any_scroll
            || pane_layout_changed
            || matches!(
                self.state.mode,
                super::model::EditorMode::Command
                    | super::model::EditorMode::Visual
                    | super::model::EditorMode::VisualLine
            );

        let needs_content_update = any_content_changed && !needs_full_render;

        if needs_full_render {
            self.view_manager.render_full(&self.state)?;
        } else if needs_content_update {
            self.view_manager.render_content_update(&self.state)?;
        } else if any_cursor_moved {
            self.view_manager.render_cursor_only(&self.state)?;
        }

        Ok(())
    }

    /// Register all default commands
    fn register_default_commands(&mut self) {
        // Movement commands
        self.commands.push(Box::new(MoveCursorLeftCommand::new()));
        self.commands.push(Box::new(MoveCursorRightCommand::new()));
        self.commands.push(Box::new(MoveCursorUpCommand::new()));
        self.commands.push(Box::new(MoveCursorDownCommand::new()));
        self.commands
            .push(Box::new(MoveCursorLineStartCommand::new()));
        self.commands
            .push(Box::new(MoveCursorLineEndCommand::new()));
        self.commands.push(Box::new(SwitchPaneCommand::new()));

        // Editing commands
        self.commands.push(Box::new(EnterInsertModeCommand::new()));
        self.commands.push(Box::new(ExitInsertModeCommand::new()));
        self.commands.push(Box::new(InsertCharCommand::new()));
        self.commands.push(Box::new(InsertNewLineCommand::new()));
        self.commands.push(Box::new(DeleteCharCommand::new()));
        self.commands.push(Box::new(EnterCommandModeCommand::new()));

        // Note: Commands are processed in order, so put more specific commands first
        // and more general commands (like InsertCharCommand) later
    }

    /// Add a custom command to the registry
    pub fn register_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Get reference to current application state (for testing/debugging)
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get mutable reference to current application state (for testing/debugging)
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

// Trait extension to allow downcasting for CommandV2 check
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Command + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TODO: Remove this trait object extension once we have a better solution
// This is a temporary workaround for the downcasting issue
impl dyn Command {
    fn as_any(&self) -> &dyn std::any::Any {
        panic!("as_any not implemented for this Command type")
    }
}
