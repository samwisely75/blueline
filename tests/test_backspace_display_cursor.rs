//! Integration test for backspace display cursor synchronization
//!
//! This test verifies that the display cursor is properly synchronized
//! with the logical cursor after backspace operations, preventing the
//! "black out" issue when the Response pane is rendered.

use blueline::repl::events::{EditorMode, LogicalPosition};
use blueline::repl::view_models::ViewModel;

#[test]
fn test_backspace_updates_display_cursor_correctly() {
    let mut vm = ViewModel::new();
    vm.change_mode(EditorMode::Insert).unwrap();

    // Insert "GET" in the request pane
    vm.insert_text("GET").unwrap();

    // Verify initial positions
    let logical_cursor = vm.get_cursor_position();
    assert_eq!(logical_cursor.line, 0);
    assert_eq!(logical_cursor.column, 3); // After "GET"

    let display_cursor = vm.get_display_cursor_position();
    assert_eq!(display_cursor.0, 0); // Display line
    assert_eq!(display_cursor.1, 3); // Display column

    // Perform backspace
    vm.delete_char_before_cursor().unwrap();

    // Verify logical cursor moved correctly
    let logical_cursor_after = vm.get_cursor_position();
    assert_eq!(logical_cursor_after.line, 0);
    assert_eq!(logical_cursor_after.column, 2); // After "GE"

    // Verify display cursor is synchronized
    let display_cursor_after = vm.get_display_cursor_position();
    assert_eq!(display_cursor_after.0, 0); // Display line
    assert_eq!(display_cursor_after.1, 2); // Display column - MUST match logical

    // Verify the text content
    assert_eq!(vm.get_request_text(), "GE");
}

#[test]
fn test_backspace_at_line_join_updates_display_cursor() {
    let mut vm = ViewModel::new();
    vm.change_mode(EditorMode::Insert).unwrap();

    // Insert multiline text
    vm.insert_text("GET /api\nHost: example.com").unwrap();

    // Position cursor at start of second line
    vm.set_cursor_position(LogicalPosition::new(1, 0)).unwrap();

    // Verify initial display cursor
    let display_cursor = vm.get_display_cursor_position();
    assert_eq!(display_cursor.0, 1); // Second display line
    assert_eq!(display_cursor.1, 0); // Start of line

    // Perform backspace (should join lines)
    vm.delete_char_before_cursor().unwrap();

    // Verify logical cursor is at end of joined first line
    let logical_cursor_after = vm.get_cursor_position();
    assert_eq!(logical_cursor_after.line, 0);
    assert_eq!(logical_cursor_after.column, 8); // After "GET /api"

    // Verify display cursor is synchronized
    let display_cursor_after = vm.get_display_cursor_position();
    assert_eq!(display_cursor_after.0, 0); // First display line
    assert_eq!(display_cursor_after.1, 8); // After "GET /api"

    // Verify the text was joined correctly
    assert_eq!(vm.get_request_text(), "GET /apiHost: example.com");
}

#[test]
fn test_multiple_backspaces_maintain_display_cursor_sync() {
    let mut vm = ViewModel::new();
    vm.change_mode(EditorMode::Insert).unwrap();

    // Insert text
    vm.insert_text("HELLO").unwrap();

    // Perform multiple backspaces
    for i in 0..3 {
        vm.delete_char_before_cursor().unwrap();

        // After each backspace, verify display cursor matches logical cursor
        let logical_cursor = vm.get_cursor_position();
        let display_cursor = vm.get_display_cursor_position();

        assert_eq!(logical_cursor.line, 0);
        assert_eq!(logical_cursor.column, 5 - (i + 1)); // 4, 3, 2
        assert_eq!(display_cursor.0, 0);
        assert_eq!(display_cursor.1, 5 - (i + 1)); // Must match logical column
    }

    // Verify final text
    assert_eq!(vm.get_request_text(), "HE");
}

#[test]
fn test_backspace_with_wrapped_lines_updates_display_cursor() {
    let mut vm = ViewModel::new();

    // Set up narrow terminal to force wrapping
    vm.update_terminal_size(20, 10);
    vm.set_wrap_enabled(true).unwrap();

    vm.change_mode(EditorMode::Insert).unwrap();

    // Insert a long line that will wrap
    vm.insert_text("This is a very long line that will wrap")
        .unwrap();

    // Get cursor position (should be at end of text)
    let logical_cursor = vm.get_cursor_position();
    assert_eq!(logical_cursor.line, 0);
    assert_eq!(logical_cursor.column, 39); // Length of text "This is a very long line that will wrap" = 39

    // Perform backspace
    vm.delete_char_before_cursor().unwrap();

    // Verify logical cursor moved back
    let logical_cursor_after = vm.get_cursor_position();
    assert_eq!(logical_cursor_after.line, 0);
    assert_eq!(logical_cursor_after.column, 38);

    // Display cursor should be on a wrapped line but properly synchronized
    let display_cursor_after = vm.get_display_cursor_position();
    // The exact display line/column depends on wrapping implementation
    // but the key is that it should not be stale/incorrect

    // Perform another backspace to ensure continued sync
    vm.delete_char_before_cursor().unwrap();

    let logical_cursor_after2 = vm.get_cursor_position();
    let display_cursor_after2 = vm.get_display_cursor_position();

    assert_eq!(logical_cursor_after2.column, 37);
    // Display cursor should have moved as well (exact position depends on wrap points)
    assert!(display_cursor_after2 != display_cursor_after);
}

#[test]
fn test_backspace_preserves_display_cursor_sync_with_response_pane() {
    let mut vm = ViewModel::new();

    // Set up response content to ensure response pane exists
    vm.set_response(
        200,
        "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"status\": \"ok\"}".to_string(),
    );

    // Make sure we're in request pane (we're already there by default)
    vm.change_mode(EditorMode::Insert).unwrap();

    // Insert text in request pane
    vm.insert_text("GET /api/test").unwrap();

    // Verify we can get display cursor position
    let display_cursor_before = vm.get_display_cursor_position();
    assert_eq!(display_cursor_before.1, 13); // After "GET /api/test"

    // Perform backspace
    vm.delete_char_before_cursor().unwrap();

    // Verify display cursor updated correctly
    let display_cursor_after = vm.get_display_cursor_position();
    assert_eq!(display_cursor_after.1, 12); // After "GET /api/tes"

    // The key test: rendering should not "black out" the response pane
    // This would happen if display cursor was out of sync
    assert_eq!(vm.get_request_text(), "GET /api/tes");
}
