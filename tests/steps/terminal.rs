//! Step definitions for terminal state and rendering
//!
//! This module contains step definitions for:
//! - Terminal content verification
//! - Line numbers and status bar
//! - Cursor appearance

use crate::common::world::BluelineWorld;
use cucumber::then;
use tracing::debug;

#[then(regex = r#"the request pane should show line number "([^"]+)" in column (\d+)"#)]
async fn then_request_pane_shows_line_number(
    world: &mut BluelineWorld,
    line_num: String,
    column: String,
) {
    debug!(
        "Verifying line number '{}' appears in column {}",
        line_num, column
    );
    let state = world.get_terminal_state().await;
    let col: usize = column.parse().expect("Invalid column number");

    // Check if the first row contains the line number at the expected position
    if let Some(first_row) = state.grid.first() {
        // Column 3 (index 2) should have the line number
        let actual_char = first_row.get(col - 1).copied().unwrap_or(' ');
        assert_eq!(
            actual_char.to_string(),
            line_num,
            "Expected line number '{line_num}' at column {column}, but found '{actual_char}'"
        );
        debug!("Line number '{}' found at column {}", line_num, column);
    } else {
        panic!("No terminal content found");
    }
}

#[then(regex = r#"the request pane should show "([^"]+)" for empty lines"#)]
async fn then_request_pane_shows_empty_lines(world: &mut BluelineWorld, marker: String) {
    debug!("Verifying empty line marker: '{}'", marker);
    let state = world.get_terminal_state().await;

    // Check rows 2 onwards for the empty line marker
    let mut found_marker = false;
    for (idx, row) in state.grid.iter().enumerate().skip(1) {
        let line: String = row.iter().collect();
        if line.contains(&marker) {
            found_marker = true;
            debug!("Found empty line marker '{}' at row {}", marker, idx + 1);
            break;
        }
    }

    assert!(
        found_marker,
        "Expected to find empty line marker '{marker}' in terminal"
    );
}

#[then(regex = r#"there should be a blinking block cursor at column (\d+)"#)]
async fn then_block_cursor_at_column(world: &mut BluelineWorld, column: String) {
    debug!("Verifying cursor at column {}", column);
    let state = world.get_terminal_state().await;
    let expected_col: u16 = column.parse::<u16>().unwrap() - 1; // Convert to 0-indexed

    assert_eq!(
        state.cursor_position.0,
        expected_col,
        "Expected cursor at column {column}, but found at column {}",
        state.cursor_position.0 + 1
    );
    debug!("Cursor confirmed at column {}", column);
}

#[then(regex = r#"^the status bar should show "([^"]+)" aligned to the right$"#)]
async fn then_status_bar_shows(world: &mut BluelineWorld, status_text: String) {
    debug!("Verifying status bar shows: '{}'", status_text);
    let state = world.get_terminal_state().await;

    // Status bar is typically on the last row
    if let Some(last_row) = state.grid.last() {
        let line: String = last_row.iter().collect();
        let trimmed = line.trim();
        assert!(
            trimmed.contains(&status_text),
            "Expected status bar to show '{status_text}', but found '{trimmed}'"
        );
        debug!("Status bar shows: '{}'", status_text);
    } else {
        panic!("No terminal content found");
    }
}

#[then(regex = r#"I should see "([^"]+)" in the status line"#)]
async fn then_should_see_in_status_line(world: &mut BluelineWorld, expected_text: String) {
    debug!("Checking status line for text: '{}'", expected_text);
    let state = world.get_terminal_state().await;

    // The status line is typically the last line of the terminal
    if let Some(last_row) = state.grid.last() {
        let line: String = last_row.iter().collect();
        let trimmed = line.trim();
        assert!(
            trimmed.contains(&expected_text) || line.contains(&expected_text),
            "Status line should contain '{expected_text}'. Status line content: '{line}'"
        );
        debug!("✅ Status line contains expected text: '{}'", expected_text);
    } else {
        // If we can't get the last line, check the full terminal content
        let terminal_content = world.get_terminal_content().await;
        assert!(
            terminal_content.contains(&expected_text),
            "Terminal should contain '{expected_text}' in status line. Full content: {terminal_content}"
        );
        debug!(
            "✅ Terminal contains expected status text: '{}'",
            expected_text
        );
    }
}

#[then("the terminal state should be valid")]
async fn then_terminal_state_should_be_valid(world: &mut BluelineWorld) {
    debug!("Verifying terminal state is valid");

    // Get terminal state and verify it's not corrupted
    let terminal_content = world.get_terminal_content().await;

    // Basic sanity checks for valid terminal state
    assert!(
        !terminal_content.is_empty(),
        "Terminal should have some content"
    );

    assert!(
        terminal_content.len() < 100_000,
        "Terminal content should not be excessively large (possible corruption)"
    );

    // Check that terminal doesn't have obvious corruption markers
    assert!(
        !terminal_content.contains("\0"),
        "Terminal should not contain null characters"
    );

    debug!("✅ Terminal state is valid");
}
