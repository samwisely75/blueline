//! # Paste Commands
//!
//! Paste commands that execute directly.
//! Demonstrates moving all business logic into the command itself.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::repl::{
    events::EditorMode,
    services::Services,
    view_models::{commands::{Command, CommandContext}, ViewModel},
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

    fn execute(&self, view_model: &mut ViewModel, services: &mut Services) -> Result<()> {
        // Get from YankService
        if let Some(yank_entry) = services.yank.paste() {
            // Paste the text after the current cursor position using type-aware paste
            view_model.paste_after_with_type(&yank_entry)?;

            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();

            // Clear any previous status message (e.g., "1 line yanked")
            view_model.clear_status_message();

            tracing::info!(
                "Pasted {} characters ({} lines) after cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            view_model.set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        Ok(())
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

    fn execute(&self, view_model: &mut ViewModel, services: &mut Services) -> Result<()> {
        // Get from YankService
        if let Some(yank_entry) = services.yank.paste() {
            tracing::debug!(
                "Retrieved yank entry with type: {:?}, text length: {}",
                yank_entry.yank_type,
                yank_entry.text.len()
            );

            // Paste the text at current position (before cursor) using type-aware paste
            view_model.paste_with_type(&yank_entry)?;

            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();

            // Clear any previous status message (e.g., "1 line yanked")
            view_model.clear_status_message();

            tracing::info!(
                "Pasted {} characters ({} lines) at cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            view_model.set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        Ok(())
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
        };

        let p_key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
        assert!(command.is_relevant(p_key, EditorMode::Normal, &context));

        // lowercase p should not be relevant
        let p_lower = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert!(!command.is_relevant(p_lower, EditorMode::Normal, &context));
    }
}