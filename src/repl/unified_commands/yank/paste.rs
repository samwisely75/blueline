//! # Paste Commands
//!
//! Paste commands following the unified command pattern.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::repl::{
    models::{events::view_events::ViewEvent, pane_state::EditorMode},
    unified_commands::{Command, CommandContext, ExecutionContext},
};

/// Paste command - pastes text after cursor (p command)
#[derive(Default)]
pub struct PasteAfterCommand;

impl PasteAfterCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for PasteAfterCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // 'p' key in Normal mode
        matches!(key_event.code, KeyCode::Char('p'))
            && key_event.modifiers.is_empty()
            && mode == EditorMode::Normal
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get from YankService
        if let Some(yank_entry) = context.services.yank.paste() {
            // Paste the text after the current cursor position using type-aware paste
            context.app_state.paste_after_with_type(&yank_entry)?;

            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();

            // Clear any previous status message (e.g., "1 line yanked")
            context.app_state.clear_status_message();

            tracing::info!(
                "Pasted {} characters ({} lines) after cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            context
                .app_state
                .set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        // Return UI update events
        Ok(vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "PasteAfterCommand"
    }
}

/// Paste command - pastes text before cursor (P command)
#[derive(Default)]
pub struct PasteBeforeCommand;

impl PasteBeforeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for PasteBeforeCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // 'P' (uppercase) key in Normal mode
        matches!(key_event.code, KeyCode::Char('P'))
            && key_event.modifiers.is_empty()
            && mode == EditorMode::Normal
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get from YankService
        if let Some(yank_entry) = context.services.yank.paste() {
            tracing::debug!(
                "Retrieved yank entry with type: {:?}, text length: {}",
                yank_entry.yank_type,
                yank_entry.text.len()
            );

            // Paste the text at current position (before cursor) using type-aware paste
            context.app_state.paste_with_type(&yank_entry)?;

            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();

            // Clear any previous status message (e.g., "1 line yanked")
            context.app_state.clear_status_message();

            tracing::info!(
                "Pasted {} characters ({} lines) at cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            context
                .app_state
                .set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        // Return UI update events
        Ok(vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "PasteBeforeCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::KeyModifiers;

    #[test]
    fn paste_after_command_should_be_relevant_for_p_in_normal_mode() {
        let command = PasteAfterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let p_key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert!(command.is_relevant(p_key, EditorMode::Normal, &context));

        // Should not be relevant in Insert mode
        assert!(!command.is_relevant(p_key, EditorMode::Insert, &context));
    }

    #[test]
    fn paste_before_command_should_be_relevant_for_uppercase_p_in_normal_mode() {
        let command = PasteBeforeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let p_key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
        assert!(command.is_relevant(p_key, EditorMode::Normal, &context));

        // lowercase p should not be relevant
        let p_lower = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert!(!command.is_relevant(p_lower, EditorMode::Normal, &context));
    }
}

// Register both commands for dynamic discovery
use crate::register_command;
register_command!(PasteAfterCommand, "PasteAfterCommand");
register_command!(PasteBeforeCommand, "PasteBeforeCommand");
