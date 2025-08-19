//! # Commands Module
//!
//! Event-driven command system with trait-based context access.
//! Commands analyze events and produce CommandEvents that describe what should happen.
//! The controller applies these events to maintain proper separation of concerns.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::repl::events::EditorMode;

// Import and re-export command event types
pub mod context;
pub mod events;

pub use context::*;
pub use events::*;

/// Command trait for event-driven architecture
pub trait Command: Send {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool;

    /// Execute command and produce events describing what should happen
    /// Commands should not mutate state directly, only produce events
    fn execute(&self, event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

/// Command trait for commands that need HTTP client access
pub trait HttpCommand: Send {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, context: &HttpCommandContext, event: &KeyEvent) -> bool;

    /// Execute command with HTTP client access
    fn execute(&self, event: KeyEvent, context: &HttpCommandContext) -> Result<Vec<CommandEvent>>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

/// Helper function to check if current mode supports navigation
/// This can be used across multiple command modules to avoid duplication
pub fn is_navigation_mode(context: &CommandContext) -> bool {
    matches!(
        context.state.current_mode,
        EditorMode::Normal | EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
    )
}

// Import command modules
pub mod app;
pub mod editing;
pub mod ex_commands;
pub mod mode;
pub mod navigation;
pub mod pane;
pub mod request;
pub mod yank;

// Re-export all commands for easy access
pub use app::AppTerminateCommand;
pub use editing::{
    DeleteCharAtCursorCommand, DeleteCharCommand, InsertCharCommand, InsertNewLineCommand,
    InsertTabCommand,
};
pub use ex_commands::{ExCommand, ExCommandRegistry};
pub use mode::{
    AppendAfterCursorCommand, AppendAtEndOfLineCommand, EnterCommandModeCommand,
    EnterInsertModeCommand, EnterVisualBlockModeCommand, EnterVisualLineModeCommand,
    EnterVisualModeCommand, ExCommandModeCommand, ExitInsertModeCommand,
    ExitVisualBlockInsertModeCommand, ExitVisualModeCommand, InsertAtBeginningOfLineCommand,
    RepeatVisualSelectionCommand, VisualBlockAppendCommand, VisualBlockInsertCommand,
};
pub use navigation::{
    BeginningOfLineCommand, EndKeyCommand, EndOfLineCommand, EndOfWordCommand, EnterGPrefixCommand,
    GoToBottomCommand, GoToTopCommand, HalfPageDownCommand, HalfPageUpCommand, HomeKeyCommand,
    MoveCursorDownCommand, MoveCursorLeftCommand, MoveCursorRightCommand, MoveCursorUpCommand,
    NextWordCommand, PageDownCommand, PageUpCommand, PreviousWordCommand, ScrollLeftCommand,
    ScrollRightCommand,
};
pub use pane::SwitchPaneCommand;
pub use request::ExecuteRequestCommand;
pub use yank::{
    ChangeSelectionCommand, CutCharacterCommand, CutSelectionCommand, DeleteSelectionCommand,
    PasteAfterCommand, PasteAtCursorCommand, YankCommand,
};

/// Type alias for command collection to reduce complexity
pub type CommandCollection = Vec<Box<dyn Command + Send>>;

/// Registry for managing all available commands
pub struct CommandRegistry {
    commands: CommandCollection,
}

impl CommandRegistry {
    /// Create new command registry with all default commands
    pub fn new() -> Self {
        let commands: CommandCollection = vec![
            // App control commands (highest priority - process first)
            Box::new(AppTerminateCommand),
            // Request commands (high priority - must intercept Enter before other commands)
            Box::new(ExecuteRequestCommand),
            // G mode commands (high priority - must be processed before regular g handling)
            Box::new(GoToTopCommand),
            Box::new(GoToBottomCommand),
            Box::new(RepeatVisualSelectionCommand), // gv command
            Box::new(EnterGPrefixCommand),
            // Scroll commands (higher priority than regular movement)
            Box::new(ScrollLeftCommand),
            Box::new(ScrollRightCommand),
            // Pagination commands (high priority - Ctrl+key combinations)
            Box::new(PageDownCommand),
            Box::new(PageUpCommand),
            Box::new(HalfPageDownCommand),
            Box::new(HalfPageUpCommand),
            // Movement commands
            Box::new(MoveCursorLeftCommand),
            Box::new(MoveCursorRightCommand),
            Box::new(MoveCursorUpCommand),
            Box::new(MoveCursorDownCommand),
            Box::new(NextWordCommand),
            Box::new(PreviousWordCommand),
            Box::new(EndOfWordCommand),
            Box::new(BeginningOfLineCommand),
            Box::new(EndOfLineCommand),
            Box::new(HomeKeyCommand),
            Box::new(EndKeyCommand),
            // Mode commands
            Box::new(EnterInsertModeCommand),
            Box::new(EnterVisualModeCommand),
            Box::new(EnterVisualLineModeCommand),
            Box::new(EnterVisualBlockModeCommand),
            Box::new(VisualBlockInsertCommand),
            Box::new(VisualBlockAppendCommand),
            Box::new(AppendAfterCursorCommand),
            Box::new(AppendAtEndOfLineCommand),
            Box::new(InsertAtBeginningOfLineCommand),
            Box::new(ExitInsertModeCommand),
            Box::new(ExitVisualBlockInsertModeCommand),
            Box::new(ExitVisualModeCommand),
            Box::new(EnterCommandModeCommand),
            Box::new(ExCommandModeCommand),
            // Pane commands
            Box::new(SwitchPaneCommand),
            // Editing commands
            Box::new(InsertCharCommand),
            Box::new(InsertNewLineCommand),
            Box::new(InsertTabCommand),
            Box::new(DeleteCharCommand),
            Box::new(DeleteCharAtCursorCommand),
            Box::new(YankCommand),
            Box::new(DeleteSelectionCommand),
            Box::new(CutSelectionCommand),
            Box::new(CutCharacterCommand),
            Box::new(ChangeSelectionCommand),
            Box::new(PasteAfterCommand),
            Box::new(PasteAtCursorCommand),
        ];

        Self { commands }
    }

    /// Find and execute the first relevant command for the given event
    /// Returns the events produced by the command that should be applied
    pub fn process_event(
        &self,
        event: KeyEvent,
        context: &CommandContext,
    ) -> Result<Vec<CommandEvent>> {
        tracing::debug!(
            "Processing key event in registry: {:?} (context: mode={:?}, pane={:?})",
            event,
            context.state.current_mode,
            context.state.current_pane
        );

        // Special logging for Enter key events to debug modifier handling
        if matches!(event.code, KeyCode::Enter) {
            tracing::warn!(
                "🔍 ENTER KEY DETECTED: modifiers={:?}, is_empty={}, mode={:?}, pane={:?}",
                event.modifiers,
                event.modifiers.is_empty(),
                context.state.current_mode,
                context.state.current_pane
            );
        }

        for (index, command) in self.commands.iter().enumerate() {
            let command_name = command.name();
            let is_relevant = command.is_relevant(context, &event);

            tracing::trace!(
                "Checking command #{}: {} -> relevant: {}",
                index,
                command_name,
                is_relevant
            );

            if is_relevant {
                tracing::info!(
                    "Found relevant command: {} (index: {})",
                    command_name,
                    index
                );
                let result = command.execute(event, context);
                match &result {
                    Ok(events) => {
                        tracing::debug!(
                            "Command {} produced {} events: {:?}",
                            command_name,
                            events.len(),
                            events
                        );
                    }
                    Err(e) => {
                        tracing::error!("Command {} execution failed: {}", command_name, e);
                    }
                }
                return result;
            }
        }

        tracing::warn!(
            "No relevant command found for event: {:?} (mode={:?}, pane={:?})",
            event,
            context.state.current_mode,
            context.state.current_pane
        );
        Ok(vec![])
    }

    /// Add a custom command to the registry
    pub fn add_command(&mut self, command: Box<dyn Command + Send>) {
        self.commands.push(command);
    }

    /// Get all commands (for debugging or introspection)
    pub fn get_commands(&self) -> &CommandCollection {
        &self.commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn create_test_context() -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: EditorMode::Normal,
                current_pane: Pane::Request,
                cursor_position: LogicalPosition { line: 0, column: 0 },
                request_text: String::new(),
                response_text: String::new(),
                terminal_dimensions: (80, 24),
                expand_tab: false,
                tab_width: 4,
            },
        }
    }

    #[test]
    fn registry_should_create_with_all_commands() {
        let registry = CommandRegistry::new();
        assert!(!registry.commands.is_empty());
        // Should have at least the core commands
        assert!(registry.commands.len() > 10);
    }

    #[test]
    fn registry_should_handle_movement_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = create_test_key_event(KeyCode::Left);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a cursor move event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested { .. }
        ));
    }

    #[test]
    fn registry_should_handle_mode_change_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = create_test_key_event(KeyCode::Char('i'));
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a mode change event to Insert mode
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::ModeChangeRequested {
                new_mode: EditorMode::Insert
            }
        );
    }

    #[test]
    fn registry_should_handle_pane_switch_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = create_test_key_event(KeyCode::Tab);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a pane switch event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::PaneSwitchRequested { .. }
        ));
    }

    #[test]
    fn registry_should_return_empty_for_unhandled_events() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = create_test_key_event(KeyCode::Char('z')); // No command for 'z' in Normal mode
        let events = registry.process_event(event, &context).unwrap();

        assert!(events.is_empty());
    }

    #[test]
    fn registry_should_respect_mode_context() {
        let registry = CommandRegistry::new();
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Insert;

        // 'i' should not be relevant in Insert mode (mode change commands are for Normal mode)
        let event = create_test_key_event(KeyCode::Char('i'));
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a character insertion event instead of mode change
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::TextInsertRequested { .. }
        ));
    }

    #[test]
    fn registry_should_allow_adding_custom_commands() {
        let mut registry = CommandRegistry::new();
        let initial_count = registry.commands.len();

        // Add a custom command (using an existing one for simplicity)
        registry.add_command(Box::new(crate::repl::commands::pane::SwitchPaneCommand));

        assert_eq!(registry.commands.len(), initial_count + 1);
    }

    #[test]
    fn registry_should_handle_ctrl_f_page_down_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a page down cursor movement event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::PageDown,
                amount: 1
            }
        ));
    }

    #[test]
    fn registry_should_not_handle_regular_f_as_page_down() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty());
        let events = registry.process_event(event, &context).unwrap();

        // Should not produce any events (regular 'f' has no command in Normal mode)
        assert!(events.is_empty());
    }

    #[test]
    fn registry_should_handle_ctrl_b_page_up_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a page up cursor movement event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::PageUp,
                amount: 1
            }
        ));
    }

    #[test]
    fn registry_should_not_handle_regular_b_as_page_up() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty());
        let events = registry.process_event(event, &context).unwrap();

        // Should produce previous word event (regular 'b' in Normal mode)
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::WordBackward,
                amount: 1
            }
        ));
    }

    #[test]
    fn registry_should_handle_ctrl_d_half_page_down_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a half page down cursor movement event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::HalfPageDown,
                amount: 1
            }
        ));
    }

    #[test]
    fn registry_should_handle_ctrl_u_half_page_up_command() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let events = registry.process_event(event, &context).unwrap();

        // Should produce a half page up cursor movement event
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::HalfPageUp,
                amount: 1
            }
        ));
    }

    #[test]
    fn registry_should_not_handle_regular_d_as_half_page_down() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let events = registry.process_event(event, &context).unwrap();

        // Should not produce any events (regular 'd' has no command in Normal mode)
        assert!(events.is_empty());
    }

    #[test]
    fn registry_should_not_handle_regular_u_as_half_page_up() {
        let registry = CommandRegistry::new();
        let context = create_test_context();

        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty());
        let events = registry.process_event(event, &context).unwrap();

        // Should not produce any events (regular 'u' has no command in Normal mode)
        assert!(events.is_empty());
    }
}
