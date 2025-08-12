//! Step definitions for line number visibility operations
//!
//! This module contains step definitions for:
//! - Line number visibility checks
//! - Line number toggle commands
//! - Content width verification

use crate::common::world::BluelineWorld;
use cucumber::{given, then};
use tracing::debug;

// === LINE NUMBER VISIBILITY ===

#[then(regex = r#"I should see line number "(\d+)" in the request pane"#)]
async fn then_should_see_line_number(world: &mut BluelineWorld, line_num: String) {
    debug!("Checking for line number '{}' in request pane", line_num);

    // Look for line number format "  1 " or " 1:" with various possible variations
    let patterns = vec![
        format!(" {} ", line_num),
        format!("{}:", line_num),
        format!(" {}:", line_num),
        format!("  {}:", line_num),
    ];

    let mut found = false;
    for pattern in &patterns {
        if world.terminal_contains(pattern).await {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "Expected to see line number '{line_num}' in request pane"
    );
}

#[then("I should not see line numbers in the request pane")]
async fn then_should_not_see_line_numbers_request(world: &mut BluelineWorld) {
    debug!("Verifying line numbers are not visible in request pane");

    // Check that common line number patterns are not present
    let has_line_numbers = world.terminal_contains("  1:").await
        || world.terminal_contains(" 1:").await
        || world.terminal_contains("1:").await
        || world.terminal_contains("  2:").await
        || world.terminal_contains(" 2:").await;

    assert!(
        !has_line_numbers,
        "Line numbers should not be visible in request pane"
    );
}

#[then("I should not see line numbers in the response pane")]
async fn then_should_not_see_line_numbers_response(world: &mut BluelineWorld) {
    debug!("Verifying line numbers are not visible in response pane");

    // Similar check for response pane
    let has_line_numbers = world.terminal_contains("  1:").await
        || world.terminal_contains(" 1:").await
        || world.terminal_contains("1:").await
        || world.terminal_contains("  2:").await
        || world.terminal_contains(" 2:").await;

    assert!(
        !has_line_numbers,
        "Line numbers should not be visible in response pane"
    );
}

// === CURSOR POSITIONING ===

#[then("the cursor should be positioned after the line number")]
async fn then_cursor_after_line_number(world: &mut BluelineWorld) {
    debug!("Verifying cursor is positioned after line number");

    // Get terminal state to check cursor position
    let state = world.get_terminal_state().await;

    // With line numbers visible, cursor should be at column 3 or greater (0-indexed)
    // (3 chars for line number + 1 space = column index 3)
    assert!(
        state.cursor_position.0 >= 3,
        "Cursor should be positioned after line number at column 3 or greater (0-indexed), but is at column {}",
        state.cursor_position.0
    );
}

#[then("the cursor should be positioned at the start of the line")]
async fn then_cursor_at_line_start(world: &mut BluelineWorld) {
    debug!("Verifying cursor is positioned at start of line");

    // Get terminal state to check cursor position
    let state = world.get_terminal_state().await;

    // Without line numbers, cursor should be at column 0
    assert_eq!(
        state.cursor_position.0, 0,
        "Cursor should be at column 0 when line numbers are hidden, but is at column {}",
        state.cursor_position.0
    );
}

// === LINE NUMBER STATE ===

#[given("line numbers are hidden")]
async fn given_line_numbers_hidden(world: &mut BluelineWorld) {
    debug!("Setting up with line numbers hidden");

    // Enter command mode and hide line numbers
    world.press_key(':').await;
    world.type_text("set number off").await;
    world.press_enter().await;
    world.tick().await.expect("Failed to tick");
}

// === CONTENT WIDTH ===

#[then("the full width of the terminal should be available for content")]
async fn then_full_width_available(world: &mut BluelineWorld) {
    debug!("Verifying full terminal width is available for content");

    // This is difficult to test directly, but we can verify that
    // the text starts at column 0 when line numbers are hidden
    let content = world.get_terminal_content().await;

    // Check that content starts at the beginning of lines
    // (no leading spaces for line numbers)
    let lines: Vec<&str> = content.lines().collect();
    for line in lines {
        if !line.is_empty() && !line.starts_with('~') {
            // Content lines should not have leading spaces when line numbers are hidden
            assert!(
                !line.starts_with("   ") && !line.starts_with("  "),
                "Content should start at beginning of line when line numbers are hidden"
            );
        }
    }
}
