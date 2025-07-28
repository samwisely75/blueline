//! # View Model Integration Tests
//!
//! Tests for the complete view model functionality including ex commands.

#[cfg(test)]
mod integration_tests {
    use crate::repl::{commands::CommandEvent, events::EditorMode, view_models::ViewModel};

    #[test]
    fn test_ex_command_mode_entry_and_exit() {
        let mut vm = ViewModel::new();

        // Start in normal mode
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), "");

        // Enter command mode
        vm.change_mode(EditorMode::Command).unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Command);

        // Add some characters
        vm.add_ex_command_char('q').unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "q");

        // Execute command (should quit and return to normal mode)
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), ""); // Buffer should be cleared
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CommandEvent::QuitRequested));
    }

    #[test]
    fn test_ex_command_quit_variations() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Test "q" command
        vm.add_ex_command_char('q').unwrap();
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CommandEvent::QuitRequested));
        assert_eq!(vm.get_mode(), EditorMode::Normal);

        // Test "q!" command
        vm.change_mode(EditorMode::Command).unwrap();
        vm.add_ex_command_char('q').unwrap();
        vm.add_ex_command_char('!').unwrap();
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CommandEvent::QuitRequested));
        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_ex_command_wrap_commands() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Test "set wrap" command
        for ch in "set wrap".chars() {
            vm.add_ex_command_char(ch).unwrap();
        }
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 0); // No command events, just internal state change
        assert_eq!(vm.get_mode(), EditorMode::Normal);

        // Test "set nowrap" command
        vm.change_mode(EditorMode::Command).unwrap();
        for ch in "set nowrap".chars() {
            vm.add_ex_command_char(ch).unwrap();
        }
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 0);
        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_ex_command_empty_command() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Execute empty command (just pressing Enter without typing)
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 0);
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), "");
    }

    #[test]
    fn test_ex_command_unknown_command() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Type unknown command
        for ch in "unknown".chars() {
            vm.add_ex_command_char(ch).unwrap();
        }
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 0); // No events for unknown commands
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), "");
    }

    #[test]
    fn test_ex_command_backspace() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Type some characters
        vm.add_ex_command_char('q').unwrap();
        vm.add_ex_command_char('u').unwrap();
        vm.add_ex_command_char('i').unwrap();
        vm.add_ex_command_char('t').unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "quit");

        // Backspace to remove last character
        vm.backspace_ex_command().unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "qui");

        // Backspace again
        vm.backspace_ex_command().unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "qu");

        // Continue backspacing until empty
        vm.backspace_ex_command().unwrap();
        vm.backspace_ex_command().unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "");

        // Backspace on empty buffer should not panic
        vm.backspace_ex_command().unwrap();
        assert_eq!(vm.get_ex_command_buffer(), "");
    }

    #[test]
    fn test_ex_command_whitespace_handling() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Test command with leading/trailing whitespace
        vm.add_ex_command_char(' ').unwrap();
        vm.add_ex_command_char('q').unwrap();
        vm.add_ex_command_char(' ').unwrap();
        assert_eq!(vm.get_ex_command_buffer(), " q ");

        // Execute - should still work due to trim()
        let events = vm.execute_ex_command().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CommandEvent::QuitRequested));
        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_delete_char_before_cursor_in_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert some text
        vm.insert_text("hello").unwrap();

        // Delete a character (should delete 'o')
        vm.delete_char_before_cursor().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 4); // Should be at position 4 (after "hell")
    }

    #[test]
    fn test_delete_char_before_cursor_at_line_start() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert simple text and test basic backspace
        vm.insert_text("hello").unwrap();

        // Move cursor to position 1
        vm.set_cursor_position(crate::repl::events::LogicalPosition::new(0, 1))
            .unwrap();

        // Delete character before cursor (should delete 'h')
        vm.delete_char_before_cursor().unwrap();

        // Should be at position 0 now
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn test_delete_char_before_cursor_only_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text in insert mode
        vm.insert_text("test").unwrap();
        // Switch to normal mode
        vm.change_mode(EditorMode::Normal).unwrap();

        // Try to delete in normal mode (should do nothing)
        vm.delete_char_before_cursor().unwrap();

        // Text should be unchanged
        assert_eq!(vm.get_request_text(), "test");
    }

    #[test]
    fn test_delete_char_after_cursor() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text and move cursor back
        vm.insert_text("hello").unwrap();
        let cursor = vm.get_cursor_position();
        vm.set_cursor_position(crate::repl::events::LogicalPosition::new(cursor.line, 2))
            .unwrap();

        // Delete character after cursor (should delete 'l')
        vm.delete_char_after_cursor().unwrap();

        // Cursor should stay at same position but text should be modified
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 2);
    }

    #[test]
    fn test_delete_char_after_cursor_at_line_end() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text and position at end
        vm.insert_text("hello").unwrap();

        // Try to delete at end of line (should do nothing)
        vm.delete_char_after_cursor().unwrap();

        // Text should be unchanged
        assert_eq!(vm.get_request_text(), "hello");
    }

    #[test]
    fn test_cursor_moves_correctly_after_backspace() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text "hello" (cursor should be at position 5)
        vm.insert_text("hello").unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 5);

        // Backspace once (should delete 'o' and move cursor to position 4)
        vm.delete_char_before_cursor().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 4);

        // Backspace again (should delete 'l' and move cursor to position 3)
        vm.delete_char_before_cursor().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 3);

        // Verify the text is now "hel"
        assert_eq!(vm.get_request_text(), "hel");
    }

    #[test]
    fn test_pane_switching() {
        let mut vm = ViewModel::new();

        // Should start in request pane
        assert_eq!(vm.get_current_pane(), crate::repl::events::Pane::Request);

        // Add a response so response pane is available
        vm.set_response(200, "test response".to_string());

        // Switch to response pane
        vm.switch_pane(crate::repl::events::Pane::Response).unwrap();
        assert_eq!(vm.get_current_pane(), crate::repl::events::Pane::Response);

        // Switch back to request pane
        vm.switch_pane(crate::repl::events::Pane::Request).unwrap();
        assert_eq!(vm.get_current_pane(), crate::repl::events::Pane::Request);
    }

    #[test]
    fn test_navigation_in_response_pane() {
        let mut vm = ViewModel::new();

        // Set up a response with multiple lines
        vm.set_response(200, "line 1\nline 2\nline 3".to_string());

        // Switch to response pane and normal mode
        vm.switch_pane(crate::repl::events::Pane::Response).unwrap();
        vm.change_mode(EditorMode::Normal).unwrap();

        assert_eq!(vm.get_current_pane(), crate::repl::events::Pane::Response);
        assert_eq!(vm.get_mode(), EditorMode::Normal);

        // Movement should work in response pane
        let initial_cursor = vm.get_cursor_position();
        assert_eq!(initial_cursor.line, 0);
        assert_eq!(initial_cursor.column, 0);

        // Try moving down
        vm.move_cursor_down().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 1); // Should move to line 1
    }

    #[test]
    fn test_command_flow_integration() {
        use crate::repl::commands::{CommandContext, CommandRegistry, ViewModelSnapshot};
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut vm = ViewModel::new();
        let registry = CommandRegistry::new();

        // Set up a response
        vm.set_response(200, "line 1\nline 2\nline 3".to_string());

        // Create context from view model state
        let context = CommandContext::new(ViewModelSnapshot::from_view_model(&vm));

        // Test Tab key (should generate PaneSwitchRequested)
        let tab_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        let events = registry.process_event(tab_event, &context).unwrap();
        assert!(!events.is_empty(), "Tab should generate events");

        // Test j key in normal mode (should generate cursor move)
        vm.change_mode(EditorMode::Normal).unwrap();
        let context = CommandContext::new(ViewModelSnapshot::from_view_model(&vm));
        let j_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        let events = registry.process_event(j_event, &context).unwrap();
        assert!(
            !events.is_empty(),
            "j key should generate cursor move events"
        );
    }
}
