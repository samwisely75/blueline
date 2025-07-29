//! # View Model Integration Tests
//!
//! Tests for the complete view model functionality including ex commands.

#[cfg(test)]
mod integration_tests {
    use crate::repl::{
        commands::CommandEvent,
        events::{EditorMode, LogicalPosition, Pane, ViewEvent},
        view_models::ViewModel,
    };

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
        vm.set_cursor_position(LogicalPosition::new(0, 1)).unwrap();

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
        vm.set_cursor_position(LogicalPosition::new(cursor.line, 2))
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
    fn test_delete_char_before_cursor_on_blank_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text with a blank line: "GET /api/users\n\n{"name": "John"}"
        vm.insert_text("GET /api/users\n\n{\"name\": \"John\"}")
            .unwrap();

        // Position cursor on the blank line (line 1, column 0)
        vm.set_cursor_position(LogicalPosition::new(1, 0)).unwrap();

        // Backspace should delete the blank line and move cursor to end of previous line
        vm.delete_char_before_cursor().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 14); // Should be at end of "GET /api/users"

        // Verify the blank line was deleted
        assert_eq!(
            vm.get_request_text(),
            "GET /api/users\n{\"name\": \"John\"}"
        );
    }

    #[test]
    fn test_delete_char_before_cursor_on_consecutive_blank_lines() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text with consecutive blank lines: "GET /api/users\n\n\n{"name": "John"}"
        vm.insert_text("GET /api/users\n\n\n{\"name\": \"John\"}")
            .unwrap();

        // Position cursor on the second blank line (line 2, column 0)
        vm.set_cursor_position(LogicalPosition::new(2, 0)).unwrap();

        // Backspace should delete only the current blank line
        vm.delete_char_before_cursor().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.column, 0); // Should be at the beginning of the first blank line

        // Verify only one blank line was deleted
        assert_eq!(
            vm.get_request_text(),
            "GET /api/users\n\n{\"name\": \"John\"}"
        );
    }

    #[test]
    fn test_delete_char_before_cursor_blank_line_moves_to_end_of_previous() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text where previous line has content
        vm.insert_text("Hello World\n").unwrap();

        // Position cursor on the blank line (line 1, column 0)
        vm.set_cursor_position(LogicalPosition::new(1, 0)).unwrap();

        // Backspace should delete the blank line and move cursor to end of "Hello World"
        vm.delete_char_before_cursor().unwrap();

        let cursor = vm.get_cursor_position();

        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 11); // Should be at end of "Hello World"

        // Verify the blank line was deleted
        assert_eq!(vm.get_request_text(), "Hello World");
    }

    #[test]
    fn test_delete_char_before_cursor_non_blank_line_joins_normally() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Insert text with non-blank second line
        vm.insert_text("Hello\nWorld").unwrap();

        // Position cursor at start of second line
        vm.set_cursor_position(LogicalPosition::new(1, 0)).unwrap();

        // Backspace should join the lines (existing behavior)
        vm.delete_char_before_cursor().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 5); // Should be after "Hello"

        // Verify lines were joined
        assert_eq!(vm.get_request_text(), "HelloWorld");
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

    // Visual mode tests
    #[test]
    fn test_visual_mode_entry_and_exit() {
        let mut vm = ViewModel::new();

        // Start in normal mode
        assert_eq!(vm.get_mode(), EditorMode::Normal);

        // Enter visual mode
        vm.change_mode(EditorMode::Visual).unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Visual);

        // Check that visual selection state is initialized
        let (start, end, pane) = vm.get_visual_selection();
        assert!(start.is_some());
        assert!(end.is_some());
        assert!(pane.is_some());
        assert_eq!(pane.unwrap(), Pane::Request);

        // Exit visual mode
        vm.change_mode(EditorMode::Normal).unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Normal);

        // Check that visual selection state is cleared
        let (start, end, pane) = vm.get_visual_selection();
        assert!(start.is_none());
        assert!(end.is_none());
        assert!(pane.is_none());
    }

    #[test]
    fn test_visual_selection_initialization() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add some text and position cursor
        vm.insert_text("hello world").unwrap();
        vm.set_cursor_position(LogicalPosition::new(0, 5)).unwrap();

        // Enter visual mode
        vm.change_mode(EditorMode::Visual).unwrap();

        // Check that selection starts and ends at cursor position
        let (start, end, pane) = vm.get_visual_selection();
        assert_eq!(start, Some(LogicalPosition::new(0, 5)));
        assert_eq!(end, Some(LogicalPosition::new(0, 5)));
        assert_eq!(pane, Some(Pane::Request));
    }

    #[test]
    fn test_visual_selection_updates_with_cursor_movement() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        // Add some text
        vm.insert_text("hello\nworld\ntest").unwrap();
        vm.set_cursor_position(LogicalPosition::new(0, 2)).unwrap();

        // Enter visual mode
        vm.change_mode(EditorMode::Visual).unwrap();

        let (start, end, _) = vm.get_visual_selection();
        assert_eq!(start, Some(LogicalPosition::new(0, 2)));
        assert_eq!(end, Some(LogicalPosition::new(0, 2)));

        // Move cursor - selection end should update
        vm.set_cursor_position(LogicalPosition::new(1, 3)).unwrap();

        let (start, end, _) = vm.get_visual_selection();
        assert_eq!(start, Some(LogicalPosition::new(0, 2))); // Start unchanged
        assert_eq!(end, Some(LogicalPosition::new(1, 3))); // End updated
    }

    #[test]
    fn test_is_position_selected_single_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();

        // Set up visual selection from column 2 to 6 on line 0
        vm.change_mode(EditorMode::Visual).unwrap();
        vm.panes[Pane::Request].visual_selection_start = Some(LogicalPosition::new(0, 2));
        vm.panes[Pane::Request].visual_selection_end = Some(LogicalPosition::new(0, 6));

        // Test positions within selection
        assert!(vm.is_position_selected(LogicalPosition::new(0, 2), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 3), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 4), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 5), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 6), Pane::Request));

        // Test positions outside selection
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 1), Pane::Request));
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 7), Pane::Request));
        assert!(!vm.is_position_selected(LogicalPosition::new(1, 3), Pane::Request));

        // Test different pane
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 3), Pane::Response));
    }

    #[test]
    fn test_is_position_selected_multi_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line1\nline2\nline3").unwrap();

        // Set up visual selection from line 0, col 2 to line 2, col 3
        vm.change_mode(EditorMode::Visual).unwrap();
        vm.panes[Pane::Request].visual_selection_start = Some(LogicalPosition::new(0, 2));
        vm.panes[Pane::Request].visual_selection_end = Some(LogicalPosition::new(2, 3));

        // Test first line (partial)
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 1), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 2), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 3), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 4), Pane::Request));

        // Test middle line (fully selected)
        assert!(vm.is_position_selected(LogicalPosition::new(1, 0), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(1, 2), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(1, 4), Pane::Request));

        // Test last line (partial)
        assert!(vm.is_position_selected(LogicalPosition::new(2, 0), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(2, 3), Pane::Request));
        assert!(!vm.is_position_selected(LogicalPosition::new(2, 4), Pane::Request));

        // Test lines outside selection
        assert!(!vm.is_position_selected(LogicalPosition::new(3, 0), Pane::Request));
    }

    #[test]
    fn test_is_position_selected_reversed_selection() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();

        // Set up visual selection with end before start (reversed)
        vm.change_mode(EditorMode::Visual).unwrap();
        vm.panes[Pane::Request].visual_selection_start = Some(LogicalPosition::new(0, 6));
        vm.panes[Pane::Request].visual_selection_end = Some(LogicalPosition::new(0, 2));

        // Should normalize selection range automatically
        assert!(vm.is_position_selected(LogicalPosition::new(0, 2), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 3), Pane::Request));
        assert!(vm.is_position_selected(LogicalPosition::new(0, 6), Pane::Request));
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 1), Pane::Request));
        assert!(!vm.is_position_selected(LogicalPosition::new(0, 7), Pane::Request));
    }

    #[test]
    fn test_visual_selection_in_response_pane() {
        let mut vm = ViewModel::new();

        // Set up response content
        vm.set_response(200, "response\ntext\nhere".to_string());
        vm.switch_pane(Pane::Response).unwrap();

        // Enter visual mode in response pane
        vm.change_mode(EditorMode::Visual).unwrap();

        let (_start, _end, pane) = vm.get_visual_selection();
        assert_eq!(pane, Some(Pane::Response));

        // Move cursor and check selection updates
        vm.set_cursor_position(LogicalPosition::new(1, 2)).unwrap();

        let (_start, end, pane) = vm.get_visual_selection();
        assert_eq!(pane, Some(Pane::Response));
        assert_eq!(end, Some(LogicalPosition::new(1, 2)));
    }

    #[test]
    fn test_visual_selection_does_not_update_across_panes() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("request text").unwrap();
        vm.set_response(200, "response text".to_string());

        // Start visual selection in request pane
        vm.change_mode(EditorMode::Visual).unwrap();
        vm.set_cursor_position(LogicalPosition::new(0, 5)).unwrap();

        // Switch to response pane - selection should not update
        vm.switch_pane(Pane::Response).unwrap();
        vm.set_cursor_position(LogicalPosition::new(0, 3)).unwrap();

        // Selection should still be in request pane and unchanged
        // With pane-based selection, the current pane (Response) has no selection
        let (_start, _end, pane) = vm.get_visual_selection();
        assert_eq!(pane, None); // Response pane has no selection

        // But the request pane should still have its original selection
        assert_eq!(
            vm.panes[Pane::Request].visual_selection_end,
            Some(LogicalPosition::new(0, 5))
        );
    }

    #[test]
    fn test_visual_mode_persists_across_cursor_movements() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello\nworld").unwrap();

        // Position cursor at a specific location before entering visual mode
        vm.set_cursor_position(LogicalPosition::new(0, 2)).unwrap();

        // Enter visual mode
        vm.change_mode(EditorMode::Visual).unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Visual);

        let (initial_start, initial_end, _) = vm.get_visual_selection();
        assert_eq!(initial_start, Some(LogicalPosition::new(0, 2)));
        assert_eq!(initial_end, Some(LogicalPosition::new(0, 2)));

        // Move cursor multiple times - should stay in visual mode
        vm.move_cursor_right().unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Visual);

        vm.move_cursor_down().unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Visual);

        vm.move_cursor_right().unwrap();
        assert_eq!(vm.get_mode(), EditorMode::Visual);

        // Selection should be updated
        let (start, end, _) = vm.get_visual_selection();
        assert!(start.is_some());
        assert!(end.is_some());
        assert_eq!(start, Some(LogicalPosition::new(0, 2))); // Start should remain unchanged
        assert_ne!(start, end); // End should be different after movements
    }

    // Tests for multiple event emission functionality
    #[test]
    fn test_emit_view_event_single_event() {
        let mut vm = ViewModel::new();

        // Emit a single event
        vm.emit_view_event([ViewEvent::StatusBarUpdateRequired]);

        // Check that it was properly collected
        let events = vm.collect_pending_view_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ViewEvent::StatusBarUpdateRequired));
    }

    #[test]
    fn test_emit_view_event_multiple_events() {
        let mut vm = ViewModel::new();

        // Emit multiple events at once
        vm.emit_view_event([
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Request,
            },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        // Check that all events were collected
        let events = vm.collect_pending_view_events();
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], ViewEvent::StatusBarUpdateRequired));
        assert!(matches!(
            events[1],
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Request
            }
        ));
        assert!(matches!(
            events[2],
            ViewEvent::PositionIndicatorUpdateRequired
        ));
    }

    #[test]
    fn test_emit_view_event_vec_of_events() {
        let mut vm = ViewModel::new();

        // Create a vector of events
        let event_vec = vec![
            ViewEvent::PaneRedrawRequired {
                pane: Pane::Request,
            },
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Response,
            },
        ];

        // Emit the vector
        vm.emit_view_event(event_vec);

        // Check that all events were collected
        let events = vm.collect_pending_view_events();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            ViewEvent::PaneRedrawRequired {
                pane: Pane::Request
            }
        ));
        assert!(matches!(
            events[1],
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Response
            }
        ));
    }

    #[test]
    fn test_emit_view_event_accumulates_events() {
        let mut vm = ViewModel::new();

        // Emit some events
        vm.emit_view_event([ViewEvent::StatusBarUpdateRequired]);
        vm.emit_view_event([
            ViewEvent::CursorUpdateRequired {
                pane: Pane::Request,
            },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        // Check that all events were accumulated
        let events = vm.collect_pending_view_events();
        assert_eq!(events.len(), 3);

        // Check that collecting clears the events
        let empty_events = vm.collect_pending_view_events();
        assert_eq!(empty_events.len(), 0);
    }
}
