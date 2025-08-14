//! # Ex Commands Module
//!
//! Implementation of ex commands (colon commands) using the command pattern.
//! This provides a unified event-driven approach for both normal and ex commands.

use anyhow::Result;

use crate::repl::commands::{
    CommandContext, CommandEvent, MovementDirection, Setting, SettingValue,
};

/// Trait for ex commands
pub trait ExCommand: Send {
    /// Parse and check if this command can handle the given ex command string
    fn can_handle(&self, command: &str) -> bool;

    /// Execute the ex command and produce events
    fn execute(&self, command: &str, context: &CommandContext) -> Result<Vec<CommandEvent>>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

/// Quit command handler (for :q and :q!)
pub struct QuitCommand;

impl ExCommand for QuitCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "q" || command == "q!"
    }

    fn execute(&self, _command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::QuitRequested])
    }

    fn name(&self) -> &'static str {
        "QuitCommand"
    }
}

/// Set wrap command handler (for :set wrap on/off)
pub struct SetWrapCommand;

impl ExCommand for SetWrapCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "set wrap on" || command == "set wrap off"
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let enable = command == "set wrap on";

        // For now, we'll create a new event type for settings changes
        // This will be handled by the controller
        Ok(vec![CommandEvent::SettingChangeRequested {
            setting: Setting::Wrap,
            value: if enable {
                SettingValue::On
            } else {
                SettingValue::Off
            },
        }])
    }

    fn name(&self) -> &'static str {
        "SetWrapCommand"
    }
}

/// Set line numbers command handler (for :set number on/off)
pub struct SetNumberCommand;

impl ExCommand for SetNumberCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "set number on" || command == "set number off"
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let enable = command == "set number on";

        Ok(vec![CommandEvent::SettingChangeRequested {
            setting: Setting::LineNumbers,
            value: if enable {
                SettingValue::On
            } else {
                SettingValue::Off
            },
        }])
    }

    fn name(&self) -> &'static str {
        "SetNumberCommand"
    }
}

/// Set clipboard integration command handler (for :set clipboard on/off)
pub struct SetClipboardCommand;

impl ExCommand for SetClipboardCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "set clipboard on" || command == "set clipboard off"
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let enable = command == "set clipboard on";

        Ok(vec![CommandEvent::SettingChangeRequested {
            setting: Setting::Clipboard,
            value: if enable {
                SettingValue::On
            } else {
                SettingValue::Off
            },
        }])
    }

    fn name(&self) -> &'static str {
        "SetClipboardCommand"
    }
}

/// Show profile command handler (for :show profile)
pub struct ShowProfileCommand;

impl ExCommand for ShowProfileCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "show profile"
    }

    fn execute(&self, _command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::ShowProfileRequested])
    }

    fn name(&self) -> &'static str {
        "ShowProfileCommand"
    }
}

/// Set tabstop command handler (for :set tabstop <number>)
pub struct SetTabstopCommand;

impl ExCommand for SetTabstopCommand {
    fn can_handle(&self, command: &str) -> bool {
        // Check if command starts with "set tabstop " followed by a number
        if let Some(value_str) = command.strip_prefix("set tabstop ") {
            value_str.parse::<usize>().is_ok()
        } else {
            false
        }
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        if let Some(value_str) = command.strip_prefix("set tabstop ") {
            if let Ok(tab_width) = value_str.parse::<usize>() {
                // Validate tab width (must be between 1 and 8)
                let tab_width = tab_width.clamp(1, 8);
                Ok(vec![CommandEvent::SettingChangeRequested {
                    setting: Setting::TabStop,
                    value: SettingValue::Number(tab_width),
                }])
            } else {
                tracing::warn!("Invalid tabstop value: {}", value_str);
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "SetTabstopCommand"
    }
}

/// Set expandtab command handler (for :set expandtab on/off)
pub struct SetExpandTabCommand;

impl ExCommand for SetExpandTabCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "set expandtab on" || command == "set expandtab off"
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let enable = command == "set expandtab on";

        Ok(vec![CommandEvent::SettingChangeRequested {
            setting: Setting::ExpandTab,
            value: if enable {
                SettingValue::On
            } else {
                SettingValue::Off
            },
        }])
    }

    fn name(&self) -> &'static str {
        "SetExpandTabCommand"
    }
}

/// Type alias to reduce complexity for ex command collection
type ExCommandCollection = Vec<Box<dyn ExCommand + Send>>;

/// Go to line command handler (for :<number>)
pub struct GoToLineCommand;

impl ExCommand for GoToLineCommand {
    fn can_handle(&self, command: &str) -> bool {
        // Check if it's a valid line number
        command.parse::<usize>().is_ok()
    }

    fn execute(&self, command: &str, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        if let Ok(line_number) = command.parse::<usize>() {
            if line_number > 0 {
                Ok(vec![CommandEvent::CursorMoveRequested {
                    direction: MovementDirection::LineNumber(line_number),
                    amount: 1,
                }])
            } else {
                tracing::warn!("Invalid line number: {}", line_number);
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "GoToLineCommand"
    }
}

/// Registry for managing ex commands
pub struct ExCommandRegistry {
    commands: ExCommandCollection,
}

impl ExCommandRegistry {
    /// Create a new ex command registry with all default commands
    pub fn new() -> Self {
        let commands: ExCommandCollection = vec![
            Box::new(QuitCommand),
            Box::new(SetWrapCommand),
            Box::new(SetNumberCommand),
            Box::new(SetClipboardCommand),
            Box::new(SetTabstopCommand),
            Box::new(SetExpandTabCommand),
            Box::new(ShowProfileCommand),
            Box::new(GoToLineCommand),
        ];

        Self { commands }
    }

    /// Parse and execute an ex command string
    pub fn execute_command(
        &self,
        command_str: &str,
        context: &CommandContext,
    ) -> Result<Vec<CommandEvent>> {
        let trimmed = command_str.trim();

        // Empty command just exits command mode
        if trimmed.is_empty() {
            return Ok(vec![]);
        }

        // Find the first command that can handle this string
        for command in &self.commands {
            if command.can_handle(trimmed) {
                tracing::debug!("Ex command '{}' handled by {}", trimmed, command.name());
                return command.execute(trimmed, context);
            }
        }

        // Unknown command
        tracing::warn!("Unknown ex command: {}", trimmed);
        Ok(vec![])
    }
}

impl Default for ExCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::ViewModelSnapshot;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};

    fn create_test_context() -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: EditorMode::Normal,
                current_pane: Pane::Request,
                cursor_position: LogicalPosition::zero(),
                request_text: String::new(),
                response_text: String::new(),
                terminal_dimensions: (80, 24),
                expand_tab: false,
                tab_width: 4,
            },
        }
    }

    #[test]
    fn quit_command_should_handle_q() {
        let cmd = QuitCommand;
        assert!(cmd.can_handle("q"));
        assert!(cmd.can_handle("q!"));
        assert!(!cmd.can_handle("quit"));
    }

    #[test]
    fn quit_command_should_produce_quit_event() {
        let cmd = QuitCommand;
        let context = create_test_context();
        let result = cmd.execute("q", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::QuitRequested);
    }

    #[test]
    fn set_wrap_command_should_handle_wrap_settings() {
        let cmd = SetWrapCommand;
        assert!(cmd.can_handle("set wrap on"));
        assert!(cmd.can_handle("set wrap off"));
        assert!(!cmd.can_handle("set wrap"));
    }

    #[test]
    fn set_tabstop_command_should_handle_tabstop_settings() {
        let cmd = SetTabstopCommand;
        assert!(cmd.can_handle("set tabstop 4"));
        assert!(cmd.can_handle("set tabstop 8"));
        assert!(cmd.can_handle("set tabstop 2"));
        assert!(!cmd.can_handle("set tabstop"));
        assert!(!cmd.can_handle("set tabstop abc"));
    }

    #[test]
    fn set_tabstop_command_should_produce_setting_change_event() {
        let cmd = SetTabstopCommand;
        let context = create_test_context();

        // Test valid tab width
        let result = cmd.execute("set tabstop 4", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            CommandEvent::SettingChangeRequested {
                setting: Setting::TabStop,
                value: SettingValue::Number(4),
            }
        );

        // Test clamping to max value
        let result = cmd.execute("set tabstop 20", &context).unwrap();
        assert_eq!(
            result[0],
            CommandEvent::SettingChangeRequested {
                setting: Setting::TabStop,
                value: SettingValue::Number(8), // Should be clamped to 8
            }
        );
    }

    #[test]
    fn set_expandtab_command_should_handle_expandtab_settings() {
        let cmd = SetExpandTabCommand;
        assert!(cmd.can_handle("set expandtab on"));
        assert!(cmd.can_handle("set expandtab off"));
        assert!(!cmd.can_handle("set expandtab"));
        assert!(!cmd.can_handle("set expandtab yes"));
    }

    #[test]
    fn set_expandtab_command_should_produce_setting_change_event() {
        let cmd = SetExpandTabCommand;
        let context = create_test_context();

        // Test enabling expandtab
        let result = cmd.execute("set expandtab on", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            CommandEvent::SettingChangeRequested {
                setting: Setting::ExpandTab,
                value: SettingValue::On,
            }
        );

        // Test disabling expandtab
        let result = cmd.execute("set expandtab off", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            CommandEvent::SettingChangeRequested {
                setting: Setting::ExpandTab,
                value: SettingValue::Off,
            }
        );
    }

    #[test]
    fn goto_line_command_should_handle_numbers() {
        let cmd = GoToLineCommand;
        assert!(cmd.can_handle("42"));
        assert!(cmd.can_handle("1"));
        assert!(!cmd.can_handle("abc"));
        assert!(!cmd.can_handle(""));
    }

    #[test]
    fn goto_line_command_should_produce_cursor_move_event() {
        let cmd = GoToLineCommand;
        let context = create_test_context();
        let result = cmd.execute("58", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::LineNumber(58),
                amount: 1,
            }
        );
    }

    #[test]
    fn registry_should_execute_known_commands() {
        let registry = ExCommandRegistry::new();
        let context = create_test_context();

        let result = registry.execute_command("q", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::QuitRequested);

        let result = registry.execute_command("show profile", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::ShowProfileRequested);
    }

    #[test]
    fn registry_should_handle_unknown_commands() {
        let registry = ExCommandRegistry::new();
        let context = create_test_context();

        let result = registry.execute_command("unknown", &context).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn registry_should_handle_empty_command() {
        let registry = ExCommandRegistry::new();
        let context = create_test_context();

        let result = registry.execute_command("", &context).unwrap();
        assert_eq!(result.len(), 0);
    }
}
