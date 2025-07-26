//! Test cases specifically for display bounds cursor movement issues
//! Ensures cursor doesn't enter status line when wrapped text fills response pane

use super::movement::*;
use super::*;
use crate::repl::model::{AppState, EditorMode, Pane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Create a test AppState with custom terminal size
fn create_test_app_state_with_size(width: u16, height: u16) -> AppState {
    AppState::new((width, height), false)
}

#[test]
fn move_cursor_down_should_not_enter_status_line_when_wrapped_text_fills_pane() {
    let mut state = create_test_app_state_with_size(80, 20);
    state.current_pane = Pane::Response;
    state.mode = EditorMode::Normal;

    // Create very long response content that will be wrapped and fill the entire response pane
    let long_line = "This is a very long line that will definitely wrap when displayed in the terminal window because it contains way more characters than the typical terminal width of 80 characters, and we want to test the edge case where wrapped content fills the entire visible response pane and scrolling stops working properly causing the cursor to enter the status line which should never happen in vim-like behavior.";
    let response_content = (0..30)
        .map(|i| format!("{} Line {}", long_line, i))
        .collect::<Vec<_>>()
        .join("\n");

    state.set_response(response_content);

    // Set response pane height to be small to trigger the issue
    state.request_pane_height = 8; // This makes response pane height around 11 lines
    let response_pane_height = state.get_response_pane_height();

    // Initialize the cache to make display lines calculations work
    let _ = state
        .cache_manager
        .update_response_cache(&state.response_buffer.as_ref().unwrap().lines, 80);

    if let Some(ref mut buffer) = state.response_buffer {
        // Position cursor at last line of content, at bottom of visible area
        buffer.scroll_offset = 20; // Show lines 20-29 (response pane height is 10)
        buffer.cursor_line = 29; // Last line of content (line 29 is at position 9 in terminal)
        buffer.cursor_col = 0;

        // Sync display positions from logical positions
        let cache = state.cache_manager.get_response_cache();
        buffer.sync_display_from_logical(&cache);

        println!("Before test:");
        println!("  Response pane height: {}", response_pane_height);
        println!("  Buffer scroll_offset: {}", buffer.scroll_offset);
        println!("  Buffer cursor_line: {}", buffer.cursor_line);
        println!("  Buffer total lines: {}", buffer.lines.len());

        let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;
        println!(
            "  Logical line position in terminal: {}",
            logical_pos_in_terminal
        );
    }

    let command = MoveCursorDownCommand;
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

    // Store state before command
    let initial_cursor_line = state.response_buffer.as_ref().unwrap().cursor_line;
    let initial_scroll_offset = state.response_buffer.as_ref().unwrap().scroll_offset;

    // Before the fix, this would cause cursor to enter status line
    // After the fix, this should either scroll to reveal more content or stay put
    let result = command.process(event, &mut state).unwrap();

    if let Some(ref buffer) = state.response_buffer {
        println!("After test:");
        println!("  Command result: {}", result);
        println!(
            "  Buffer scroll_offset: {} (was {})",
            buffer.scroll_offset, initial_scroll_offset
        );
        println!(
            "  Buffer cursor_line: {} (was {})",
            buffer.cursor_line, initial_cursor_line
        );
        println!("  Buffer total lines: {}", buffer.lines.len());

        // The key check: cursor position should be valid relative to terminal display
        let logical_line_in_terminal = buffer.cursor_line - buffer.scroll_offset;
        println!(
            "  Logical line position in terminal: {}",
            logical_line_in_terminal
        );

        // This is the critical assertion: cursor should not be positioned
        // beyond the response pane height (which would put it in status line)
        assert!(
            logical_line_in_terminal < response_pane_height,
            "Cursor is at logical line {} in terminal (cursor_line {} - scroll_offset {}), \
             but response pane height is only {}. This would put cursor in status line!",
            logical_line_in_terminal,
            buffer.cursor_line,
            buffer.scroll_offset,
            response_pane_height
        );

        // Cursor should never be positioned beyond the available content
        assert!(buffer.cursor_line < buffer.lines.len());

        // Most importantly: cursor should never be beyond the last line of content
        assert!(
            buffer.cursor_line < buffer.lines.len(),
            "Cursor line {} should be less than total lines {}",
            buffer.cursor_line,
            buffer.lines.len()
        );
    }
}

#[test]
fn move_cursor_down_should_handle_display_cache_bounds_correctly() {
    let mut state = create_test_app_state_with_size(40, 15); // Smaller terminal for easier testing
    state.current_pane = Pane::Response;
    state.mode = EditorMode::Normal;

    // Create content with some lines that will wrap
    let content = [
        "Short line",
        "This is a much longer line that will definitely wrap in a 40-character terminal width",
        "Another short line",
        "Final line",
    ]
    .join("\n");

    state.set_response(content);

    if let Some(ref mut buffer) = state.response_buffer {
        // Position at last logical line
        buffer.cursor_line = buffer.lines.len() - 1;
        buffer.cursor_col = 0;
        buffer.scroll_offset = 0;

        // Sync display positions from logical positions
        let cache = state.cache_manager.get_response_cache();
        buffer.sync_display_from_logical(&cache);
    }

    let command = MoveCursorDownCommand;
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

    let result = command.process(event, &mut state).unwrap();

    // Should not move beyond last line
    if let Some(ref buffer) = state.response_buffer {
        assert_eq!(buffer.cursor_line, buffer.lines.len() - 1);
        assert!(!result); // Command should return false since no movement occurred
    }
}

#[test]
fn move_cursor_down_should_scroll_when_more_content_available() {
    let mut state = create_test_app_state_with_size(80, 20);
    state.current_pane = Pane::Response;
    state.mode = EditorMode::Normal;

    // Create content with many lines
    let content = (0..50)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    state.set_response(content);

    let response_pane_height = 10;
    state.request_pane_height = 9; // Makes response pane around 10 lines

    if let Some(ref mut buffer) = state.response_buffer {
        // Position at bottom of visible area but with more content below
        buffer.scroll_offset = 20;
        buffer.cursor_line = buffer.scroll_offset + response_pane_height - 1; // At bottom of visible area
        buffer.cursor_col = 0;

        // Sync display positions from logical positions
        let cache = state.cache_manager.get_response_cache();
        buffer.sync_display_from_logical(&cache);
    }

    let command = MoveCursorDownCommand;
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

    let initial_scroll = state.response_buffer.as_ref().unwrap().scroll_offset;
    let initial_cursor = state.response_buffer.as_ref().unwrap().cursor_line;

    let result = command.process(event, &mut state).unwrap();

    // Should either move cursor or scroll to reveal more content
    assert!(result);

    if let Some(ref buffer) = state.response_buffer {
        let moved_cursor = buffer.cursor_line != initial_cursor;
        let scrolled = buffer.scroll_offset != initial_scroll;

        // Either cursor moved or we scrolled (or both)
        assert!(moved_cursor || scrolled);

        // Cursor should still be valid
        assert!(buffer.cursor_line < buffer.lines.len());
    }
}

#[test]
fn move_cursor_down_should_not_move_beyond_terminal_display_bounds_with_wrapping() {
    let mut state = create_test_app_state_with_size(40, 12); // Small terminal to force wrapping
    state.current_pane = Pane::Response;
    state.mode = EditorMode::Normal;

    // Create content where wrapped lines could cause terminal boundary issues
    let content = [
        "Line 1: Short",
        "Line 2: This is a very long line that will definitely wrap in a 40-character terminal width, creating multiple display lines",
        "Line 3: Another long line that wraps around and takes multiple display lines in the terminal window",
        "Line 4: Short",
        "Line 5: Short",
    ].join("\n");

    state.set_response(content);

    // Set response pane height to be very small to trigger boundary issues
    state.request_pane_height = 8; // This makes response pane height only 3 lines
    let response_pane_height = state.get_response_pane_height();

    // Update cache with correct terminal width
    let _ = state
        .cache_manager
        .update_response_cache(&state.response_buffer.as_ref().unwrap().lines, 40);

    if let Some(ref mut buffer) = state.response_buffer {
        // Position near bottom where wrapped content might cause issues
        buffer.scroll_offset = 0;
        buffer.cursor_line = 1; // On the long line that wraps
        buffer.cursor_col = 0;

        // Sync display positions from logical positions
        let cache = state.cache_manager.get_response_cache();
        buffer.sync_display_from_logical(&cache);

        println!("Before test:");
        println!(
            "  Terminal size: {}x{}",
            state.terminal_size.0, state.terminal_size.1
        );
        println!("  Response pane height: {}", response_pane_height);
        println!("  Buffer scroll_offset: {}", buffer.scroll_offset);
        println!("  Buffer cursor_line: {}", buffer.cursor_line);
        println!("  Buffer total lines: {}", buffer.lines.len());

        let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;
        println!(
            "  Logical line position in terminal: {}",
            logical_pos_in_terminal
        );
    }

    let command = MoveCursorDownCommand;

    // Try multiple cursor movements to see if we can get beyond bounds
    for i in 0..10 {
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        if let Some(ref buffer) = state.response_buffer {
            let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;

            println!(
                "After movement {}: cursor_line={}, scroll_offset={}, terminal_pos={}, result={}",
                i + 1,
                buffer.cursor_line,
                buffer.scroll_offset,
                logical_pos_in_terminal,
                result
            );

            // Critical check: cursor should never be positioned beyond response pane bounds
            if logical_pos_in_terminal >= response_pane_height {
                panic!(
                    "BOUNDARY VIOLATION: After movement {}, cursor is at terminal position {} \
                       but response pane height is only {}. This puts cursor in status line!",
                    i + 1,
                    logical_pos_in_terminal,
                    response_pane_height
                );
            }

            // If command returned false, we shouldn't keep trying
            if !result {
                println!("Movement {} returned false, stopping", i + 1);
                break;
            }
        }
    }

    // Final verification
    if let Some(ref buffer) = state.response_buffer {
        let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;

        assert!(
            logical_pos_in_terminal < response_pane_height,
            "Final cursor position {} is beyond response pane height {}",
            logical_pos_in_terminal,
            response_pane_height
        );

        assert!(buffer.cursor_line < buffer.lines.len());
    }
}

#[test]
fn fallback_movement_should_not_position_cursor_beyond_terminal_bounds() {
    let mut state = create_test_app_state_with_size(80, 10); // Small terminal for testing
    state.current_pane = Pane::Response;
    state.mode = EditorMode::Normal;

    // Create content that will trigger fallback logic (no wrapped lines to avoid display cache)
    let content = (0..20)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    state.set_response(content);

    // Set a very small response pane height to trigger the boundary issue
    state.request_pane_height = 7; // This makes response pane height only 2 lines
    let response_pane_height = state.get_response_pane_height();

    // DON'T update the cache to force fallback logic
    // (or corrupt the cache somehow)

    if let Some(ref mut buffer) = state.response_buffer {
        // Position cursor to trigger the problematic scenario
        buffer.scroll_offset = 10;
        buffer.cursor_line = 10; // At bottom of visible area
        buffer.cursor_col = 0;

        println!("Test setup:");
        println!(
            "  Terminal size: {}x{}",
            state.terminal_size.0, state.terminal_size.1
        );
        println!("  Response pane height: {}", response_pane_height);
        println!("  Buffer scroll_offset: {}", buffer.scroll_offset);
        println!("  Buffer cursor_line: {}", buffer.cursor_line);
        println!("  Buffer total lines: {}", buffer.lines.len());

        let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;
        println!(
            "  Initial logical line position in terminal: {}",
            logical_pos_in_terminal
        );
    }

    let command = MoveCursorDownCommand;
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

    // This should trigger the problematic fallback logic
    let result = command.process(event, &mut state).unwrap();

    if let Some(ref buffer) = state.response_buffer {
        let logical_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;

        println!("After movement:");
        println!("  Command result: {}", result);
        println!("  Buffer scroll_offset: {}", buffer.scroll_offset);
        println!("  Buffer cursor_line: {}", buffer.cursor_line);
        println!(
            "  Logical line position in terminal: {}",
            logical_pos_in_terminal
        );

        // This is the key test: after fallback movement, cursor should not be beyond terminal bounds
        if logical_pos_in_terminal >= response_pane_height {
            panic!(
                "FALLBACK BOUNDARY VIOLATION: Cursor is at terminal position {} \
                   but response pane height is only {}. This puts cursor in status line!\
                   cursor_line={}, scroll_offset={}",
                logical_pos_in_terminal,
                response_pane_height,
                buffer.cursor_line,
                buffer.scroll_offset
            );
        }

        assert!(
            logical_pos_in_terminal < response_pane_height,
            "Cursor terminal position {} exceeds response pane height {}",
            logical_pos_in_terminal,
            response_pane_height
        );
    }
}
