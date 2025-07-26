//! # MVVM Command Pattern Implementation
//!
//! Clean command pattern that delegates to ViewModel methods.
//! Commands are thin wrappers that map input events to business logic.

use crate::mvvm::events::{EditorMode, Pane};
use crate::mvvm::view_models::ViewModel;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Command trait for MVVM pattern
///
/// Commands check relevancy and delegate to ViewModel methods.
/// This keeps commands simple and business logic centralized.
pub trait Command {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool;

    /// Execute command by delegating to ViewModel
    fn execute(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

// =================================================================
// Movement Commands
// =================================================================

/// Move cursor left (h key or left arrow)
pub struct MoveCursorLeftCommand;

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('h') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Left => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_left()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

/// Move cursor right (l key or right arrow)
pub struct MoveCursorRightCommand;

impl Command for MoveCursorRightCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('l') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Right => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_right()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorRight"
    }
}

/// Move cursor up (k key or up arrow)
pub struct MoveCursorUpCommand;

impl Command for MoveCursorUpCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('k') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Up => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_up()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorUp"
    }
}

/// Move cursor down (j key or down arrow)
pub struct MoveCursorDownCommand;

impl Command for MoveCursorDownCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('j') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Down => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_down()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorDown"
    }
}

// =================================================================
// Mode Change Commands
// =================================================================

/// Enter insert mode (i key)
pub struct EnterInsertModeCommand;

impl Command for EnterInsertModeCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('i'))
            && view_model.get_mode() == EditorMode::Normal
            && view_model.get_current_pane() == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.change_mode(EditorMode::Insert)?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "EnterInsertMode"
    }
}

/// Exit insert mode (Escape key)
pub struct ExitInsertModeCommand;

impl Command for ExitInsertModeCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Esc) && view_model.get_mode() == EditorMode::Insert
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.change_mode(EditorMode::Normal)?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ExitInsertMode"
    }
}

/// Enter command mode (: key)
pub struct EnterCommandModeCommand;

impl Command for EnterCommandModeCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char(':'))
            && view_model.get_mode() == EditorMode::Normal
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.change_mode(EditorMode::Command)?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "EnterCommandMode"
    }
}

// =================================================================
// Pane Commands
// =================================================================

/// Switch between panes (Tab key)
pub struct SwitchPaneCommand;

impl Command for SwitchPaneCommand {
    fn is_relevant(&self, _view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Tab)
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        let new_pane = match view_model.get_current_pane() {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        };
        view_model.switch_pane(new_pane)?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
    }
}

// =================================================================
// Text Editing Commands
// =================================================================

/// Insert character in insert mode
pub struct InsertCharCommand;

impl Command for InsertCharCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(ch) => {
                !event.modifiers.contains(KeyModifiers::CONTROL)
                    && (ch.is_ascii_graphic() || ch == ' ')
                    && view_model.get_mode() == EditorMode::Insert
                    && view_model.get_current_pane() == Pane::Request
            }
            _ => false,
        }
    }

    fn execute(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        if let KeyCode::Char(ch) = event.code {
            view_model.insert_char(ch)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "InsertChar"
    }
}

/// Insert new line (Enter in insert mode)
pub struct InsertNewLineCommand;

impl Command for InsertNewLineCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Enter)
            && view_model.get_mode() == EditorMode::Insert
            && view_model.get_current_pane() == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.insert_text("\n")?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "InsertNewLine"
    }
}

/// Delete character before cursor (Backspace in insert mode)
pub struct DeleteCharCommand;

impl Command for DeleteCharCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Backspace)
            && view_model.get_mode() == EditorMode::Insert
            && view_model.get_current_pane() == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.delete_char_before_cursor()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "DeleteChar"
    }
}

// =================================================================
// HTTP Commands
// =================================================================

/// Execute HTTP request (Enter in normal mode)
pub struct ExecuteRequestCommand;

impl Command for ExecuteRequestCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Enter)
            && view_model.get_mode() == EditorMode::Normal
            && view_model.get_current_pane() == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.execute_request()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ExecuteRequest"
    }
}

// =================================================================
// Command Registry
// =================================================================

/// Type alias for command collection to reduce complexity
pub type CommandCollection = Vec<Box<dyn Command>>;

/// Registry that holds all available commands
pub struct CommandRegistry {
    commands: CommandCollection,
}

impl CommandRegistry {
    /// Create new command registry with default commands
    pub fn new() -> Self {
        let mut registry = Self {
            commands: Vec::new(),
        };

        registry.register_default_commands();
        registry
    }

    /// Register all default commands
    fn register_default_commands(&mut self) {
        // Movement commands
        self.add_command(Box::new(MoveCursorLeftCommand));
        self.add_command(Box::new(MoveCursorRightCommand));
        self.add_command(Box::new(MoveCursorUpCommand));
        self.add_command(Box::new(MoveCursorDownCommand));

        // Mode commands
        self.add_command(Box::new(EnterInsertModeCommand));
        self.add_command(Box::new(ExitInsertModeCommand));
        self.add_command(Box::new(EnterCommandModeCommand));

        // Pane commands
        self.add_command(Box::new(SwitchPaneCommand));

        // Text editing commands
        self.add_command(Box::new(InsertCharCommand));
        self.add_command(Box::new(InsertNewLineCommand));
        self.add_command(Box::new(DeleteCharCommand));

        // HTTP commands
        self.add_command(Box::new(ExecuteRequestCommand));
    }

    /// Add a command to the registry
    pub fn add_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Process a key event through all commands
    pub fn process_event(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        for command in &self.commands {
            if command.is_relevant(view_model, &event) {
                return command.execute(event, view_model);
            }
        }
        Ok(false) // No command handled the event
    }

    /// Get all commands (for testing/debugging)
    pub fn commands(&self) -> &CommandCollection {
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

    fn create_test_view_model() -> ViewModel {
        ViewModel::new()
    }

    #[test]
    fn move_cursor_left_should_be_relevant_in_normal_mode() {
        let view_model = create_test_view_model();
        let command = MoveCursorLeftCommand;
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(&view_model, &event));
    }

    #[test]
    fn move_cursor_left_should_execute() {
        let mut view_model = create_test_view_model();
        view_model.change_mode(EditorMode::Insert).unwrap();
        view_model.insert_text("hello").unwrap();
        view_model.change_mode(EditorMode::Normal).unwrap();

        let command = MoveCursorLeftCommand;
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        let result = command.execute(event, &mut view_model).unwrap();
        assert!(result);

        // Cursor should have moved left
        let cursor = view_model.get_cursor_position();
        assert_eq!(cursor.column, 4); // Was at 5, now at 4
    }

    #[test]
    fn enter_insert_mode_should_be_relevant() {
        let view_model = create_test_view_model();
        let command = EnterInsertModeCommand;
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        assert!(command.is_relevant(&view_model, &event));
    }

    #[test]
    fn enter_insert_mode_should_execute() {
        let mut view_model = create_test_view_model();
        let command = EnterInsertModeCommand;
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        let result = command.execute(event, &mut view_model).unwrap();
        assert!(result);
        assert_eq!(view_model.get_mode(), EditorMode::Insert);
    }

    #[test]
    fn insert_char_should_be_relevant_in_insert_mode() {
        let mut view_model = create_test_view_model();
        view_model.change_mode(EditorMode::Insert).unwrap();

        let command = InsertCharCommand;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(command.is_relevant(&view_model, &event));
    }

    #[test]
    fn insert_char_should_execute() {
        let mut view_model = create_test_view_model();
        view_model.change_mode(EditorMode::Insert).unwrap();

        let command = InsertCharCommand;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        let result = command.execute(event, &mut view_model).unwrap();
        assert!(result);

        let text = view_model.get_request_text();
        assert_eq!(text, "a");
    }

    #[test]
    fn switch_pane_should_execute() {
        let mut view_model = create_test_view_model();
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        assert!(command.is_relevant(&view_model, &event));

        let result = command.execute(event, &mut view_model).unwrap();
        assert!(result);
        assert_eq!(view_model.get_current_pane(), Pane::Response);
    }

    #[test]
    fn command_registry_should_process_events() {
        let mut view_model = create_test_view_model();
        let registry = CommandRegistry::new();

        // Test entering insert mode
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        let result = registry.process_event(event, &mut view_model).unwrap();
        assert!(result);
        assert_eq!(view_model.get_mode(), EditorMode::Insert);

        // Test inserting character
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let result = registry.process_event(event, &mut view_model).unwrap();
        assert!(result);
        assert_eq!(view_model.get_request_text(), "a");
    }

    #[test]
    fn command_registry_should_return_false_for_unknown_events() {
        let mut view_model = create_test_view_model();
        let registry = CommandRegistry::new();

        // Test unknown key combination
        let event = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let result = registry.process_event(event, &mut view_model).unwrap();
        assert!(!result); // Should not be handled
    }
}
