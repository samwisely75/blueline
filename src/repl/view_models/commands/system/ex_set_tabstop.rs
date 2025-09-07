//! # Ex Set Tabstop Command
//!
//! Handles `:set tabstop=N` and `:set tabstop!` ex commands for configuring tab width.
//! This command supports both explicit numeric values and toggling between common values.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::models::settings::{Setting, SettingValue};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set tabstop ex commands (:set tabstop=N, :set tabstop!)
pub struct ExSetTabstopCommand;

impl Command for ExSetTabstopCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains tabstop commands
        let buffer = context.ex_command_buffer.trim();

        // Support :set tabstop! (toggle)
        if buffer == "set tabstop!" {
            return true;
        }

        // Support :set tabstop N and :set tabstop=N formats
        if let Some(rest) = buffer.strip_prefix("set tabstop") {
            // :set tabstop N (space-separated - must start with space)
            if rest.starts_with(' ') {
                let value_part = rest.trim();
                return value_part.parse::<usize>().is_ok();
            }

            // :set tabstop=N (equals sign - must start with equals directly)
            if let Some(value_str) = rest.strip_prefix("=") {
                return value_str.trim().parse::<usize>().is_ok();
            }
        }

        false
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set tabstop!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Handle toggle command
                let current_tabstop = context.app_state.pane_manager().get_tab_width();
                let new_tabstop = if current_tabstop == 4 { 8 } else { 4 };

                context
                    .app_state
                    .apply_setting(Setting::TabStop, SettingValue::Number(new_tabstop))?;
                context
                    .app_state
                    .set_status_message(format!("Tab stop toggled to {new_tabstop}"));

                tracing::info!("Tab stop toggled from {current_tabstop} to {new_tabstop}");

                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            }
            _ => {
                // Handle numeric value commands by checking if it starts with "set tabstop"
                if let Some(rest) = command.strip_prefix("set tabstop") {
                    let rest = rest.trim();

                    // Try to parse numeric value (handle both "N" and "=N" formats)
                    let value_str = if let Some(equals_part) = rest.strip_prefix("=") {
                        equals_part.trim()
                    } else {
                        rest
                    };

                    // Parse the value first to avoid borrowing issues
                    if let Ok(tab_width) = value_str.parse::<usize>() {
                        // Clear the command buffer and exit command mode
                        context.app_state.clear_ex_command_buffer();
                        let previous_mode = context.app_state.get_previous_mode();
                        context.app_state.change_mode(previous_mode)?;

                        // Validate tab width (must be between 1 and 8)
                        let tab_width = tab_width.clamp(1, 8);

                        context
                            .app_state
                            .apply_setting(Setting::TabStop, SettingValue::Number(tab_width))?;
                        context
                            .app_state
                            .set_status_message(format!("Tab stop set to {tab_width}"));

                        tracing::info!("Tab stop set to {tab_width}");

                        Ok(vec![
                            PostCommandAction::CurrentAreaRedrawRequired,
                            PostCommandAction::StatusBarUpdateRequired,
                        ])
                    } else {
                        // Clone the invalid value before any mutations
                        let invalid_value = value_str.to_string();

                        // Clear the command buffer and exit command mode for error case too
                        context.app_state.clear_ex_command_buffer();
                        let previous_mode = context.app_state.get_previous_mode();
                        context.app_state.change_mode(previous_mode)?;

                        context
                            .app_state
                            .set_status_message(format!("Invalid tabstop value: {invalid_value}"));
                        tracing::warn!("Invalid tabstop value: {invalid_value}");

                        Ok(vec![PostCommandAction::StatusBarUpdateRequired])
                    }
                } else {
                    // This shouldn't happen given our is_relevant check, but handle gracefully
                    tracing::warn!(
                        "ExSetTabstopCommand executed with unexpected buffer: {command}"
                    );
                    Ok(vec![])
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetTabstopCommand"
    }
}

// Register the command
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExSetTabstopCommand",
        factory: || Box::new(ExSetTabstopCommand),
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn ex_set_tabstop_command_should_return_correct_name() {
        let command = ExSetTabstopCommand;
        assert_eq!(command.name(), "ExSetTabstopCommand");
    }

    #[test]
    fn is_relevant_should_detect_tabstop_numeric_commands() {
        let command = ExSetTabstopCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        // Test space-separated format
        let context = CommandContext {
            ex_command_buffer: "set tabstop 4".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));

        // Test equals format
        let context = CommandContext {
            ex_command_buffer: "set tabstop=8".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_detect_tabstop_toggle_command() {
        let command = ExSetTabstopCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let context = CommandContext {
            ex_command_buffer: "set tabstop!".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_reject_invalid_commands() {
        let command = ExSetTabstopCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        // Invalid numeric value
        let context = CommandContext {
            ex_command_buffer: "set tabstop abc".to_string(),
            ..create_test_context()
        };
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));

        // Wrong mode
        let context = CommandContext {
            ex_command_buffer: "set tabstop 4".to_string(),
            ..create_test_context()
        };
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Wrong key
        let wrong_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert!(!command.is_relevant(wrong_key, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_support_various_valid_formats() {
        let command = ExSetTabstopCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let test_cases = vec![
            "set tabstop 1",
            "set tabstop 2",
            "set tabstop 4",
            "set tabstop 8",
            "set tabstop=1",
            "set tabstop=2",
            "set tabstop=4",
            "set tabstop=8",
            "set tabstop!",
        ];

        for buffer in test_cases {
            let context = CommandContext {
                ex_command_buffer: buffer.to_string(),
                ..create_test_context()
            };

            assert!(
                command.is_relevant(key_event, EditorMode::Command, &context),
                "Should be relevant for buffer: '{buffer}'"
            );
        }
    }

    #[test]
    fn is_relevant_should_reject_invalid_formats() {
        let command = ExSetTabstopCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let test_cases = vec![
            "set tabstop",     // No value
            "set tabstop abc", // Non-numeric
            "set tabstop =4",  // Space before equals
            "w",               // Different command
            "set wrap on",     // Different setting
            "tabstop 4",       // Missing "set"
            "",                // Empty
        ];

        for buffer in test_cases {
            let context = CommandContext {
                ex_command_buffer: buffer.to_string(),
                ..create_test_context()
            };

            assert!(
                !command.is_relevant(key_event, EditorMode::Command, &context),
                "Should not be relevant for buffer: '{buffer}'"
            );
        }
    }
}
