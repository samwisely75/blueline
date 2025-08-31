//! # Setting Change Command
//!
//! Command to handle setting changes from ex commands and configuration.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::repl::commands::events::{Setting, SettingValue};
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to handle setting changes
///
/// This command processes setting changes such as:
/// - Clipboard integration (via YankService)
/// - Tab settings (tabstop, expandtab)
/// - Display settings (wrap, line numbers)
pub struct SettingChangeCommand {
    setting: Setting,
    value: SettingValue,
}

impl SettingChangeCommand {
    /// Create new SettingChangeCommand
    pub fn new(setting: Setting, value: SettingValue) -> Self {
        Self { setting, value }
    }
}

impl Command for SettingChangeCommand {
    fn is_relevant(
        &self,
        _key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // This command is not triggered by key events directly
        // It's triggered by CommandEvent::SettingChangeRequested from ex commands
        false
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Handle clipboard setting through YankService
        if self.setting == Setting::Clipboard {
            let enable = self.value == SettingValue::On;
            context.services.yank.set_clipboard_enabled(enable)?;

            // Update status message
            let message = if enable {
                "Clipboard integration enabled"
            } else {
                "Clipboard integration disabled"
            };
            context.app_state.set_status_message(message.to_string());

            tracing::info!(
                "Clipboard integration {}",
                if enable { "enabled" } else { "disabled" }
            );

            Ok(vec![ViewEvent::StatusBarUpdateRequired])
        } else {
            // Other settings go through AppState
            context.app_state.apply_setting(self.setting, self.value)?;

            // Generate status message based on setting type
            let message = match (&self.setting, &self.value) {
                (Setting::Wrap, SettingValue::On) => "Line wrapping enabled",
                (Setting::Wrap, SettingValue::Off) => "Line wrapping disabled",
                (Setting::LineNumbers, SettingValue::On) => "Line numbers enabled",
                (Setting::LineNumbers, SettingValue::Off) => "Line numbers disabled",
                (Setting::ExpandTab, SettingValue::On) => {
                    "Expand tab enabled (spaces will be used)"
                }
                (Setting::ExpandTab, SettingValue::Off) => {
                    "Expand tab disabled (tabs will be used)"
                }
                (Setting::TabStop, SettingValue::Number(n)) => {
                    context
                        .app_state
                        .set_status_message(format!("Tab stop set to {n}"));
                    return Ok(vec![
                        ViewEvent::CurrentAreaRedrawRequired,
                        ViewEvent::StatusBarUpdateRequired,
                    ]);
                }
                _ => {
                    context.app_state.set_status_message(format!(
                        "Setting {:?} changed to {:?}",
                        self.setting, self.value
                    ));
                    return Ok(vec![ViewEvent::StatusBarUpdateRequired]);
                }
            };

            context.app_state.set_status_message(message.to_string());

            tracing::info!("Setting {:?} changed to {:?}", self.setting, self.value);

            // Most settings require redraw
            Ok(vec![
                ViewEvent::CurrentAreaRedrawRequired,
                ViewEvent::StatusBarUpdateRequired,
            ])
        }
    }

    fn name(&self) -> &'static str {
        "SettingChangeCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

    #[test]
    fn setting_change_command_should_return_correct_name() {
        let command = SettingChangeCommand::new(Setting::Wrap, SettingValue::On);
        assert_eq!(command.name(), "SettingChangeCommand");
    }

    #[test]
    fn setting_change_command_should_not_be_relevant_for_key_events() {
        use crate::repl::models::pane_state::Pane;
        use crossterm::event::{KeyCode, KeyModifiers};

        let command = SettingChangeCommand::new(Setting::Wrap, SettingValue::On);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Should never be relevant for key events
        let any_key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        assert!(!command.is_relevant(any_key, EditorMode::Normal, &context));
    }

    #[test]
    fn setting_change_command_should_handle_clipboard_setting() {
        let command = SettingChangeCommand::new(Setting::Clipboard, SettingValue::On);
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ViewEvent::StatusBarUpdateRequired));
    }

    #[test]
    fn setting_change_command_should_handle_wrap_setting() {
        let command = SettingChangeCommand::new(Setting::Wrap, SettingValue::Off);
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], ViewEvent::CurrentAreaRedrawRequired));
        assert!(matches!(events[1], ViewEvent::StatusBarUpdateRequired));
    }

    #[test]
    fn setting_change_command_should_handle_tabstop_setting() {
        let command = SettingChangeCommand::new(Setting::TabStop, SettingValue::Number(4));
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], ViewEvent::CurrentAreaRedrawRequired));
        assert!(matches!(events[1], ViewEvent::StatusBarUpdateRequired));
    }
}
