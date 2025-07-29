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

    #[test]
    fn test_line_navigation_with_valid_line_number() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add multi-line content to test navigation
        vm.insert_text("line 1\nline 2\nline 3\nline 4").unwrap();
        vm.change_mode(EditorMode::Command).unwrap();

        // Navigate to line 2 using :2 command
        vm.add_ex_command_char('2').unwrap();
        let events = vm.execute_ex_command().unwrap();

        // Should get a cursor move event
        assert_eq!(events.len(), 1);
        if let CommandEvent::CursorMoveRequested { direction, amount } = &events[0] {
            assert_eq!(*amount, 1);
            if let crate::repl::commands::MovementDirection::LineNumber(line_num) = direction {
                assert_eq!(*line_num, 2);
            } else {
                panic!(
                    "Expected LineNumber movement direction, got {:?}",
                    direction
                );
            }
        } else {
            panic!("Expected CursorMoveRequested event, got {:?}", events[0]);
        }

        // Should be back in normal mode
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), ""); // Buffer should be cleared
    }

    #[test]
    fn test_line_navigation_with_line_number_one() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add multi-line content
        vm.insert_text("line 1\nline 2\nline 3").unwrap();
        vm.change_mode(EditorMode::Command).unwrap();

        // Navigate to line 1 using :1 command
        vm.add_ex_command_char('1').unwrap();
        let events = vm.execute_ex_command().unwrap();

        // Should get a cursor move event
        assert_eq!(events.len(), 1);
        if let CommandEvent::CursorMoveRequested {
            direction,
            amount: _,
        } = &events[0]
        {
            if let crate::repl::commands::MovementDirection::LineNumber(line_num) = direction {
                assert_eq!(*line_num, 1);
            } else {
                panic!("Expected LineNumber movement direction");
            }
        } else {
            panic!("Expected CursorMoveRequested event");
        }

        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_line_navigation_with_zero_line_number_should_be_ignored() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Command).unwrap();

        // Try to navigate to line 0 - should be ignored
        vm.add_ex_command_char('0').unwrap();
        let events = vm.execute_ex_command().unwrap();

        // Should not generate any cursor move events (line 0 is invalid)
        assert_eq!(events.len(), 0);
        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_line_navigation_with_large_line_number() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add only 2 lines
        vm.insert_text("line 1\nline 2").unwrap();
        vm.change_mode(EditorMode::Command).unwrap();

        // Try to navigate to line 100 - should work (cursor will be clamped)
        vm.add_ex_command_char('1').unwrap();
        vm.add_ex_command_char('0').unwrap();
        vm.add_ex_command_char('0').unwrap();
        let events = vm.execute_ex_command().unwrap();

        // Should get a cursor move event for line 100
        assert_eq!(events.len(), 1);
        if let CommandEvent::CursorMoveRequested {
            direction,
            amount: _,
        } = &events[0]
        {
            if let crate::repl::commands::MovementDirection::LineNumber(line_num) = direction {
                assert_eq!(*line_num, 100);
            } else {
                panic!("Expected LineNumber movement direction");
            }
        } else {
            panic!("Expected CursorMoveRequested event");
        }

        assert_eq!(vm.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn test_move_cursor_to_line_method_with_valid_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add content with multiple lines
        vm.insert_text("line 1\nline 2\nline 3\nline 4\nline 5")
            .unwrap();

        // Move to line 3
        vm.move_cursor_to_line(3).unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 2); // 0-based indexing, so line 3 = index 2
        assert_eq!(cursor.column, 0); // Should be at beginning of line
    }

    #[test]
    fn test_move_cursor_to_line_method_with_line_beyond_buffer() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add only 2 lines
        vm.insert_text("line 1\nline 2").unwrap();

        // Try to move to line 10 (beyond buffer)
        vm.move_cursor_to_line(10).unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 1); // Should be clamped to last line (0-based)
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn test_move_cursor_to_line_method_with_zero_should_do_nothing() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        vm.insert_text("line 1\nline 2").unwrap();

        // Get initial cursor position
        let initial_cursor = vm.get_cursor_position();

        // Try to move to line 0 (invalid)
        vm.move_cursor_to_line(0).unwrap();

        // Cursor should not have moved
        let final_cursor = vm.get_cursor_position();
        assert_eq!(initial_cursor, final_cursor);
    }

    #[test]
    fn test_show_profile_command() {
        let mut vm = ViewModel::new();

        // Set custom profile info
        vm.set_profile_info(
            "test-profile".to_string(),
            "/custom/path/profile".to_string(),
        );

        // Enter command mode and execute :show profile
        vm.change_mode(EditorMode::Command).unwrap();
        vm.add_ex_command_char('s').unwrap();
        vm.add_ex_command_char('h').unwrap();
        vm.add_ex_command_char('o').unwrap();
        vm.add_ex_command_char('w').unwrap();
        vm.add_ex_command_char(' ').unwrap();
        vm.add_ex_command_char('p').unwrap();
        vm.add_ex_command_char('r').unwrap();
        vm.add_ex_command_char('o').unwrap();
        vm.add_ex_command_char('f').unwrap();
        vm.add_ex_command_char('i').unwrap();
        vm.add_ex_command_char('l').unwrap();
        vm.add_ex_command_char('e').unwrap();

        let events = vm.execute_ex_command().unwrap();

        // Should get a ShowProfileRequested event
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], CommandEvent::ShowProfileRequested));

        // Should be back in normal mode
        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_ex_command_buffer(), ""); // Buffer should be cleared
    }

    #[test]
    fn test_profile_info_getters_and_setters() {
        let mut vm = ViewModel::new();

        // Check default values
        assert_eq!(vm.get_profile_name(), "default");
        assert_eq!(vm.get_profile_path(), "~/.blueline/profile");

        // Set custom values
        vm.set_profile_info("custom-profile".to_string(), "/custom/path".to_string());

        // Check updated values
        assert_eq!(vm.get_profile_name(), "custom-profile");
        assert_eq!(vm.get_profile_path(), "/custom/path");
    }

    #[test]
    fn test_status_message_functionality() {
        let mut vm = ViewModel::new();

        // Initially no status message
        assert_eq!(vm.get_status_message(), None);

        // Set a status message
        vm.set_status_message("Test message");
        assert_eq!(vm.get_status_message(), Some("Test message"));

        // Set another message
        vm.set_status_message("Another message".to_string());
        assert_eq!(vm.get_status_message(), Some("Another message"));

        // Clear the message
        vm.clear_status_message();
        assert_eq!(vm.get_status_message(), None);
    }
}
