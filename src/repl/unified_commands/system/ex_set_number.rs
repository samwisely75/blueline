//! # Ex Set Number Command
//!
//! Handles `:set number on`, `:set number off` and `:set number!` (toggle) ex commands
//! for controlling line number display settings.
//! This follows the unified command system approach for ex commands.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set number ex commands (:set number on/off, :set number!)
pub struct ExSetNumberCommand;

impl Command for ExSetNumberCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains number commands
        let buffer = context.ex_command_buffer.trim();
        matches!(buffer, "set number on" | "set number off" | "set number!")
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set number on" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Enable line numbers
                context
                    .app_state
                    .pane_manager
                    .set_line_numbers_visible(true);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();
                context.app_state.set_status_message("Line numbers enabled");

                tracing::info!("Line numbers enabled");

                let mut events = vec![
                    PostCommandAction::FullRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ];
                events.extend(visibility_events);
                Ok(events)
            }
            "set number off" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Disable line numbers
                context
                    .app_state
                    .pane_manager
                    .set_line_numbers_visible(false);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();
                context
                    .app_state
                    .set_status_message("Line numbers disabled");

                tracing::info!("Line numbers disabled");

                let mut events = vec![
                    PostCommandAction::FullRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ];
                events.extend(visibility_events);
                Ok(events)
            }
            "set number!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Toggle line numbers state
                let current_visible = context.app_state.pane_manager.is_line_numbers_visible();
                context
                    .app_state
                    .pane_manager
                    .set_line_numbers_visible(!current_visible);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();

                let status_msg = if current_visible {
                    "Line numbers disabled"
                } else {
                    "Line numbers enabled"
                };
                context.app_state.set_status_message(status_msg);

                tracing::info!(
                    "Line numbers toggled from {} to {}",
                    current_visible,
                    !current_visible
                );

                let mut events = vec![
                    PostCommandAction::FullRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ];
                events.extend(visibility_events);
                Ok(events)
            }
            _ => {
                // This shouldn't happen given our is_relevant check, but handle gracefully
                tracing::warn!(
                    "ExSetNumberCommand executed with unexpected buffer: {}",
                    command
                );
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetNumberCommand"
    }
}

// Register the command
inventory::submit!(
    crate::repl::unified_commands::dynamic_registry::CommandEntry {
        name: "ExSetNumberCommand",
        factory: || Box::new(ExSetNumberCommand),
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
    fn ex_set_number_command_should_return_correct_name() {
        let command = ExSetNumberCommand;
        assert_eq!(command.name(), "ExSetNumberCommand");
    }

    #[test]
    fn is_relevant_should_detect_number_on_command() {
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let context = CommandContext {
            ex_command_buffer: "set number on".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_detect_number_off_command() {
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let context = CommandContext {
            ex_command_buffer: "set number off".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_detect_number_toggle_command() {
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let context = CommandContext {
            ex_command_buffer: "set number!".to_string(),
            ..create_test_context()
        };
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_reject_invalid_commands() {
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        // Wrong command
        let context = CommandContext {
            ex_command_buffer: "set wrap on".to_string(),
            ..create_test_context()
        };
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));

        // Wrong mode
        let context = CommandContext {
            ex_command_buffer: "set number on".to_string(),
            ..create_test_context()
        };
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Wrong key
        let wrong_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let context = CommandContext {
            ex_command_buffer: "set number on".to_string(),
            ..create_test_context()
        };
        assert!(!command.is_relevant(wrong_key, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_support_all_valid_formats() {
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let test_cases = vec!["set number on", "set number off", "set number!"];

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
        let command = ExSetNumberCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());

        let test_cases = vec![
            "set number",     // No on/off/!
            "number on",      // Missing "set"
            "set wrap on",    // Different setting
            "set number yes", // Invalid value
            "w",              // Different command
            "",               // Empty
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
