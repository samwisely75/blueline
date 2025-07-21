//! Unit tests for REPL command processing
//!
//! Tests command execution, key handling, and state transitions
//! without requiring actual terminal interaction.

#[cfg(test)]
mod command_tests {
    use crate::repl::command::Command;
    use crate::repl::commands::editing::{ExitInsertModeCommand, InsertCharCommand};
    use crate::repl::commands::movement::{MoveCursorLeftCommand, SwitchPaneCommand};
    use crate::repl::model::{EditorMode, Pane};
    use crate::repl::testing::{MockWriter, ReplTestHelper};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_insert_char_command() {
        let mut helper = ReplTestHelper::new();
        let command = InsertCharCommand::new();

        // Create a key event for typing 'a'
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        // Set to insert mode first
        helper.model.mode = EditorMode::Insert;

        // Execute the command
        let result = command.process(key_event, &mut helper.model);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true (handled)

        // Verify character was inserted
        let request_content = helper.get_request_content();
        assert!(request_content.contains('a'));
    }

    #[test]
    fn test_exit_insert_mode_command() {
        let mut helper = ReplTestHelper::new();
        let command = ExitInsertModeCommand::new();

        // Create escape key event
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // Set model to insert mode first
        helper.model.mode = EditorMode::Insert;

        // Execute the command
        let result = command.process(key_event, &mut helper.model);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify mode changed to normal
        assert_eq!(helper.model.mode, EditorMode::Normal);
    }

    #[test]
    fn test_switch_pane_command() {
        let mut helper = ReplTestHelper::new();
        let command = SwitchPaneCommand::new();

        // Initially should be in Request pane
        assert_eq!(helper.get_active_pane(), Pane::Request);

        // Create Ctrl+W event
        let ctrl_w_event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        // Execute first part (Ctrl+W)
        let result = command.process(ctrl_w_event, &mut helper.model);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Execute second part (w)
        let w_event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        let result = command.process(w_event, &mut helper.model);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Should switch to Response pane
        assert_eq!(helper.get_active_pane(), Pane::Response);
    }

    #[test]
    fn test_command_relevance_checking() {
        let helper = ReplTestHelper::new();

        // Test insert command relevance (only relevant in Insert mode)
        let insert_command = InsertCharCommand::new();
        assert!(!insert_command.is_relevant(&helper.model)); // Should be false in Normal mode

        // Test movement command relevance (only relevant in Normal mode)
        let move_command = MoveCursorLeftCommand::new();
        assert!(move_command.is_relevant(&helper.model)); // Should be true in Normal mode

        // Test pane switch command relevance
        let switch_command = SwitchPaneCommand::new();
        assert!(switch_command.is_relevant(&helper.model)); // Should be true in Normal mode
    }

    #[test]
    fn test_mock_writer_output_capture() {
        let mut writer = MockWriter::new();

        // Write some test data
        use std::io::Write;
        write!(writer, "Test output: {}", 42).unwrap();

        let output = writer.get_output();
        assert_eq!(output, "Test output: 42");

        // Clear and verify
        writer.clear();
        assert_eq!(writer.get_output(), "");
    }

    #[test]
    fn test_helper_basic_setup() {
        let helper = ReplTestHelper::new();
        assert_eq!(helper.get_active_pane(), Pane::Request);
        assert_eq!(helper.get_request_content(), "");
        assert_eq!(helper.get_response_content(), "");
    }

    #[test]
    fn test_event_sequence_building() {
        let mut helper = ReplTestHelper::new();

        // Create a sequence: type "hello"
        helper.type_text("hello").press_enter().press_escape();

        // Verify events were added
        assert!(helper.has_events());

        // Count the events: 'h' + 'e' + 'l' + 'l' + 'o' + ENTER + ESC = 7 events
        let mut event_count = 0;
        while helper.has_events() {
            helper.next_event();
            event_count += 1;
        }
        assert_eq!(event_count, 7);
    }

    #[test]
    fn test_colon_command_sequence() {
        let mut helper = ReplTestHelper::new();

        // Create a colon command sequence
        helper.enter_colon_command("send");

        // Verify events were added
        assert!(helper.has_events());

        // Should have: ':' + 's' + 'e' + 'n' + 'd' + ENTER = 6 events
        let mut event_count = 0;
        while helper.has_events() {
            helper.next_event();
            event_count += 1;
        }
        assert_eq!(event_count, 6);
    }
}
