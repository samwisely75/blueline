//! # ExQuitCommand Tests
//!
//! Test the ex command design implementation using the unified command system.

#[cfg(test)]
mod tests {
    use super::super::ex_quit::ExQuitCommand;
    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crate::repl::unified_commands::{Command, CommandContext};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn ex_quit_command_should_be_relevant_for_enter_in_command_mode_with_q() {
        let command = ExQuitCommand;

        // Test with ":q" in command buffer
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(
            command.is_relevant(enter_key, EditorMode::Command, &context),
            "Should be relevant for Enter key in Command mode with 'q' buffer"
        );
    }

    #[test]
    fn ex_quit_command_should_be_relevant_for_enter_in_command_mode_with_q_force() {
        let command = ExQuitCommand;

        // Test with ":q!" in command buffer
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q!".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(
            command.is_relevant(enter_key, EditorMode::Command, &context),
            "Should be relevant for Enter key in Command mode with 'q!' buffer"
        );
    }

    #[test]
    fn ex_quit_command_should_not_be_relevant_for_other_modes() {
        let command = ExQuitCommand;

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Test different modes
        assert!(
            !command.is_relevant(enter_key, EditorMode::Normal, &context),
            "Should not be relevant in Normal mode"
        );
        assert!(
            !command.is_relevant(enter_key, EditorMode::Insert, &context),
            "Should not be relevant in Insert mode"
        );
        assert!(
            !command.is_relevant(enter_key, EditorMode::Visual, &context),
            "Should not be relevant in Visual mode"
        );
    }

    #[test]
    fn ex_quit_command_should_not_be_relevant_for_other_keys() {
        let command = ExQuitCommand;

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q".to_string(),
        };

        // Test different keys
        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(
            !command.is_relevant(space_key, EditorMode::Command, &context),
            "Should not be relevant for space key"
        );

        let escape_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(
            !command.is_relevant(escape_key, EditorMode::Command, &context),
            "Should not be relevant for escape key"
        );
    }

    #[test]
    fn ex_quit_command_should_not_be_relevant_for_other_commands() {
        let command = ExQuitCommand;

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Test with different command buffers
        let test_cases = vec!["w", "wq", "set wrap on", "123", "", "quit"];

        for buffer in test_cases {
            let context = CommandContext {
                current_mode: EditorMode::Command,
                current_pane: Pane::Request,
                is_read_only: false,
                has_selection: false,
                ex_command_buffer: buffer.to_string(),
            };

            assert!(
                !command.is_relevant(enter_key, EditorMode::Command, &context),
                "Should not be relevant for buffer: '{buffer}'"
            );
        }
    }

    #[test]
    fn ex_quit_command_name_should_be_correct() {
        let command = ExQuitCommand;
        assert_eq!(command.name(), "ExQuitCommand");
    }
}
