//! # Ex Set Expandtab Command
//!
//! Handles `:set expandtab on/off` and `:set expandtab!` (toggle) ex commands for
//! configuring tab expansion behavior. When expandtab is enabled, tab characters
//! are converted to spaces using the current tabstop width.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::models::settings::{Setting, SettingValue};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set expandtab ex commands (:set expandtab on/off/!)
pub struct ExSetExpandtabCommand;

impl Command for ExSetExpandtabCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains set expandtab commands
        let buffer = context.ex_command_buffer.trim();
        matches!(
            buffer,
            "set expandtab on" | "set expandtab off" | "set expandtab!"
        )
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set expandtab on" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Apply setting via AppState
                context
                    .app_state
                    .apply_setting(Setting::ExpandTab, SettingValue::On)?;

                // Set status message
                context
                    .app_state
                    .set_status_message("Expand tab enabled (spaces will be used)".to_string());

                tracing::info!("Expandtab enabled");

                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            }
            "set expandtab off" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Apply setting via AppState
                context
                    .app_state
                    .apply_setting(Setting::ExpandTab, SettingValue::Off)?;

                // Set status message
                context
                    .app_state
                    .set_status_message("Expand tab disabled (tabs will be used)".to_string());

                tracing::info!("Expandtab disabled");

                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            }
            "set expandtab!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Toggle expandtab setting
                let current_value = context.app_state.pane_manager().get_expand_tab();
                let new_value = if current_value {
                    SettingValue::Off
                } else {
                    SettingValue::On
                };

                // Apply setting via AppState
                context
                    .app_state
                    .apply_setting(Setting::ExpandTab, new_value)?;

                // Set appropriate status message
                let message = if current_value {
                    "Expand tab disabled (tabs will be used)"
                } else {
                    "Expand tab enabled (spaces will be used)"
                };
                context.app_state.set_status_message(message.to_string());

                tracing::info!(
                    "Expandtab toggled to {}",
                    if current_value { "disabled" } else { "enabled" }
                );

                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            }
            _ => {
                // This shouldn't happen given our is_relevant check, but handle gracefully
                tracing::warn!(
                    "ExSetExpandtabCommand executed with unexpected buffer: {}",
                    command
                );
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetExpandtabCommand"
    }
}

// Register the command
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExSetExpandtabCommand",
        factory: || Box::new(ExSetExpandtabCommand),
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::app_state::AppState;
    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    fn create_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    #[test]
    fn ex_set_expandtab_command_should_return_correct_name() {
        let command = ExSetExpandtabCommand;
        assert_eq!(command.name(), "ExSetExpandtabCommand");
    }

    #[test]
    fn is_relevant_should_match_expandtab_on_command() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set expandtab on".to_string();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_match_expandtab_off_command() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set expandtab off".to_string();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_match_expandtab_toggle_command() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set expandtab!".to_string();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_not_match_other_commands() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set tabstop 4".to_string();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_not_match_wrong_key() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set expandtab on".to_string();

        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_not_match_wrong_mode() {
        let command = ExSetExpandtabCommand;
        let mut context = create_test_context();
        context.ex_command_buffer = "set expandtab on".to_string();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_should_enable_expandtab() {
        let command = ExSetExpandtabCommand;
        let (mut app_state, mut services) = create_execution_context();

        // Set up the command buffer
        app_state.set_ex_command_buffer("set expandtab on".to_string());
        app_state.change_mode(EditorMode::Command).unwrap();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);
        assert!(matches!(
            actions[0],
            PostCommandAction::CurrentAreaRedrawRequired
        ));
        assert!(matches!(
            actions[1],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Verify expandtab is enabled
        assert!(execution_context.app_state.pane_manager().get_expand_tab());

        // Verify command buffer is cleared
        assert!(execution_context
            .app_state
            .get_ex_command_buffer()
            .is_empty());

        // Verify mode is restored
        assert_ne!(execution_context.app_state.get_mode(), EditorMode::Command);
    }

    #[test]
    fn execute_should_disable_expandtab() {
        let command = ExSetExpandtabCommand;
        let (mut app_state, mut services) = create_execution_context();

        // First enable expandtab
        app_state
            .apply_setting(Setting::ExpandTab, SettingValue::On)
            .unwrap();

        // Set up the command buffer
        app_state.set_ex_command_buffer("set expandtab off".to_string());
        app_state.change_mode(EditorMode::Command).unwrap();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);

        // Verify expandtab is disabled
        assert!(!execution_context.app_state.pane_manager().get_expand_tab());
    }

    #[test]
    fn execute_should_toggle_expandtab_from_enabled_to_disabled() {
        let command = ExSetExpandtabCommand;
        let (mut app_state, mut services) = create_execution_context();

        // First enable expandtab
        app_state
            .apply_setting(Setting::ExpandTab, SettingValue::On)
            .unwrap();

        // Set up the toggle command buffer
        app_state.set_ex_command_buffer("set expandtab!".to_string());
        app_state.change_mode(EditorMode::Command).unwrap();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);

        // Verify expandtab is now disabled (toggled from enabled)
        assert!(!execution_context.app_state.pane_manager().get_expand_tab());
    }

    #[test]
    fn execute_should_toggle_expandtab_from_disabled_to_enabled() {
        let command = ExSetExpandtabCommand;
        let (mut app_state, mut services) = create_execution_context();

        // Ensure expandtab starts disabled (default state)
        assert!(!app_state.pane_manager().get_expand_tab());

        // Set up the toggle command buffer
        app_state.set_ex_command_buffer("set expandtab!".to_string());
        app_state.change_mode(EditorMode::Command).unwrap();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);

        // Verify expandtab is now enabled (toggled from disabled)
        assert!(execution_context.app_state.pane_manager().get_expand_tab());
    }

    #[test]
    fn execute_should_handle_unknown_command_gracefully() {
        let command = ExSetExpandtabCommand;
        let (mut app_state, mut services) = create_execution_context();

        // Set up an unexpected command (though this shouldn't happen in practice)
        app_state.set_ex_command_buffer("set expandtab unknown".to_string());

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 0); // No actions for unknown command
    }

    #[test]
    fn execute_should_set_appropriate_status_messages() {
        let command = ExSetExpandtabCommand;

        // Test "on" message
        let (mut app_state, mut services) = create_execution_context();
        app_state.set_ex_command_buffer("set expandtab on".to_string());
        app_state.change_mode(EditorMode::Command).unwrap();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context,
            )
            .unwrap();
        let status = execution_context.app_state.get_status_message().unwrap();
        assert!(status.contains("enabled") && status.contains("spaces"));

        // Test "off" message
        let (mut app_state2, mut services2) = create_execution_context();
        app_state2.set_ex_command_buffer("set expandtab off".to_string());
        app_state2.change_mode(EditorMode::Command).unwrap();

        let mut execution_context2 = ExecutionContext {
            app_state: &mut app_state2,
            services: &mut services2,
        };

        command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context2,
            )
            .unwrap();
        let status2 = execution_context2.app_state.get_status_message().unwrap();
        assert!(status2.contains("disabled") && status2.contains("tabs"));
    }
}
