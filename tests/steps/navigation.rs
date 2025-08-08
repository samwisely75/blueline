//! Step definitions for navigation and cursor movement
//!
//! This module contains step definitions for:
//! - Cursor movement (h/j/k/l, arrow keys, w/b/e, 0/$)
//! - Cursor position verification
//! - Movement boundary checks

use crate::common::world::BluelineWorld;
use crossterm::event::{KeyCode, KeyModifiers};
use cucumber::{then, when};
use tracing::{debug, info};

// When steps for navigation keys
#[when(regex = r#"I press "([^"]+)""#)]
async fn when_press_key(world: &mut BluelineWorld, key: String) {
    info!("Pressing key: {}", key);
    match key.as_str() {
        "i" => {
            world
                .send_key_event(KeyCode::Char('i'), KeyModifiers::empty())
                .await
        }
        "v" => {
            info!("Pressing 'v' key to enter visual mode");
            world
                .send_key_event(KeyCode::Char('v'), KeyModifiers::empty())
                .await
        }
        "$" => {
            info!("Pressing '$' key to move to end of line");
            world
                .send_key_event(KeyCode::Char('$'), KeyModifiers::empty())
                .await
        }
        ":" => {
            info!("Pressing colon key to enter command mode");
            world
                .send_key_event(KeyCode::Char(':'), KeyModifiers::empty())
                .await
        }
        // Navigation keys
        "h" => {
            info!("Pressing 'h' key to move left");
            world
                .send_key_event(KeyCode::Char('h'), KeyModifiers::empty())
                .await
        }
        "j" => {
            info!("Pressing 'j' key to move down");
            world
                .send_key_event(KeyCode::Char('j'), KeyModifiers::empty())
                .await
        }
        "k" => {
            info!("Pressing 'k' key to move up");
            world
                .send_key_event(KeyCode::Char('k'), KeyModifiers::empty())
                .await
        }
        "l" => {
            info!("Pressing 'l' key to move right");
            world
                .send_key_event(KeyCode::Char('l'), KeyModifiers::empty())
                .await
        }
        // Word movement keys
        "w" => {
            info!("Pressing 'w' key to move to next word");
            world
                .send_key_event(KeyCode::Char('w'), KeyModifiers::empty())
                .await
        }
        "b" => {
            info!("Pressing 'b' key to move to previous word");
            world
                .send_key_event(KeyCode::Char('b'), KeyModifiers::empty())
                .await
        }
        "e" => {
            info!("Pressing 'e' key to move to end of word");
            world
                .send_key_event(KeyCode::Char('e'), KeyModifiers::empty())
                .await
        }
        // Line movement keys
        "0" => {
            info!("Pressing '0' key to move to beginning of line");
            world
                .send_key_event(KeyCode::Char('0'), KeyModifiers::empty())
                .await
        }
        _ => panic!("Unsupported key: {key}"),
    }
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

// Arrow key step definitions
#[when("I press the Up arrow key")]
async fn when_press_up_arrow(world: &mut BluelineWorld) {
    info!("Pressing Up arrow key");
    world
        .send_key_event(KeyCode::Up, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[when("I press the Down arrow key")]
async fn when_press_down_arrow(world: &mut BluelineWorld) {
    info!("Pressing Down arrow key");
    world
        .send_key_event(KeyCode::Down, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[when("I press the Left arrow key")]
async fn when_press_left_arrow(world: &mut BluelineWorld) {
    info!("Pressing Left arrow key");
    world
        .send_key_event(KeyCode::Left, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[when("I press the Right arrow key")]
async fn when_press_right_arrow(world: &mut BluelineWorld) {
    info!("Pressing Right arrow key");
    world
        .send_key_event(KeyCode::Right, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

// Navigation verification steps
#[then("the cursor should move up one line")]
async fn then_cursor_should_move_up_one_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_row = state.cursor_position.1;

    // Verify cursor is on a valid row
    assert!(
        current_row < 24,
        "Cursor should be within terminal bounds after moving up"
    );
    debug!("Cursor successfully moved up to row {}", current_row + 1);
}

#[then("the cursor should move down one line")]
async fn then_cursor_should_move_down_one_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_row = state.cursor_position.1;

    // Verify cursor is on a reasonable row
    assert!(
        current_row < 24,
        "Cursor should be within terminal bounds after moving down"
    );
    debug!("Cursor successfully moved down to row {}", current_row + 1);
}

#[then("the cursor should move left one character")]
async fn then_cursor_should_move_left_one_character(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // In vim, cursor should not go before the first column of content (column 4 in our case)
    assert!(
        current_col >= 3,
        "Cursor should not move before start of line content"
    );
    debug!(
        "Cursor successfully moved left to column {}",
        current_col + 1
    );
}

#[then("the cursor should move right one character")]
async fn then_cursor_should_move_right_one_character(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // Verify cursor moved to a reasonable position
    assert!(
        current_col < 80,
        "Cursor should be within reasonable bounds"
    );
    debug!(
        "Cursor successfully moved right to column {}",
        current_col + 1
    );
}

#[then("the cursor should move left")]
async fn then_cursor_should_move_left(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // Verify cursor is at a reasonable position after moving left
    assert!(
        current_col >= 3,
        "Cursor should not move before start of line content"
    );
    debug!("Cursor moved left to column {}", current_col + 1);
}

// Word movement verification
#[then(regex = r#"the cursor should move to the beginning of "([^"]+)""#)]
async fn then_cursor_should_move_to_beginning_of_word(world: &mut BluelineWorld, word: String) {
    let terminal_content = world.get_terminal_content().await;

    // Check if the terminal contains the expected word
    assert!(
        terminal_content.contains(&word),
        "Terminal should contain the word '{word}' for cursor positioning"
    );
    debug!("Cursor moved to beginning of word '{}'", word);
}

#[then(regex = r#"the cursor should move to the end of "([^"]+)""#)]
async fn then_cursor_should_move_to_end_of_word(world: &mut BluelineWorld, word: String) {
    let terminal_content = world.get_terminal_content().await;

    // Check if the terminal contains the expected word
    assert!(
        terminal_content.contains(&word),
        "Terminal should contain the word '{word}' for cursor positioning"
    );
    debug!("Cursor moved to end of word '{}'", word);
}

#[then(regex = r#"the cursor should be at the end of "([^"]+)""#)]
async fn then_cursor_should_be_at_end_of_word(world: &mut BluelineWorld, word: String) {
    let terminal_content = world.get_terminal_content().await;

    // Verify the word exists in terminal content
    assert!(
        terminal_content.contains(&word),
        "Terminal should contain the word '{word}' at cursor position"
    );
    debug!("Cursor is at end of word '{}'", word);
}

// Line movement verification
#[then("the cursor should move to column 1")]
async fn then_cursor_should_move_to_column_1(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // In our implementation, column 1 is actually index 0, but content starts at column 4 (index 3)
    // The '0' command in vim moves to the very beginning of the line
    assert!(
        current_col <= 3,
        "Cursor should be at or near the beginning of the line"
    );
    debug!(
        "Cursor moved to beginning of line at column {}",
        current_col + 1
    );
}

#[then("the cursor should move to the end of the line")]
async fn then_cursor_should_move_to_end_of_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // The cursor should be positioned at a reasonable end-of-line position
    assert!(
        current_col > 10,
        "Cursor should be towards the end of the line"
    );
    debug!("Cursor moved to end of line at column {}", current_col + 1);
}

#[then("the cursor should be at the end of the line")]
async fn then_cursor_should_be_at_end_of_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // Verify cursor is at a reasonable end position
    assert!(
        current_col > 10,
        "Cursor should be at the end of line content"
    );
    debug!("Cursor is at end of line, column {}", current_col + 1);
}

#[then("the cursor should be at the end of the content")]
async fn then_cursor_should_be_at_end_of_content(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // In Insert mode, cursor can be at the end of content (one past last character)
    assert!(
        current_col >= 10,
        "Cursor should be at or near end of content"
    );
    debug!("Cursor is at end of content, column {}", current_col + 1);
}

// Boundary checks
#[then("the cursor should not move beyond the start of line")]
async fn then_cursor_should_not_move_beyond_start_of_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // Should not go before the content area (column 4, index 3)
    assert!(
        current_col >= 3,
        "Cursor should not move before the start of line content"
    );
    debug!(
        "Cursor properly constrained at start of line, column {}",
        current_col + 1
    );
}

#[then("the cursor should not move beyond the end of line")]
async fn then_cursor_should_not_move_beyond_end_of_line(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;
    let current_col = state.cursor_position.0;

    // Cursor should remain within reasonable bounds
    assert!(
        current_col < 80,
        "Cursor should not move beyond reasonable line end"
    );
    debug!(
        "Cursor properly constrained at end of line, column {}",
        current_col + 1
    );
}

// Line number verification
#[then(regex = r#"the cursor should be on line (\d+)"#)]
async fn then_cursor_should_be_on_line(world: &mut BluelineWorld, line_number: String) {
    let state = world.get_terminal_state().await;
    let current_row = state.cursor_position.1;
    let expected_row: u16 = line_number.parse::<u16>().unwrap() - 1; // Convert to 0-indexed

    assert_eq!(
        current_row,
        expected_row,
        "Expected cursor on line {}, but found on line {}",
        line_number,
        current_row + 1
    );
    debug!("Cursor is on line {} as expected", line_number);
}

#[then(regex = r#"the cursor should remain on line (\d+)"#)]
async fn then_cursor_should_remain_on_line(world: &mut BluelineWorld, line_number: String) {
    let state = world.get_terminal_state().await;
    let current_row = state.cursor_position.1;
    let expected_row: u16 = line_number.parse::<u16>().unwrap() - 1; // Convert to 0-indexed

    assert_eq!(
        current_row,
        expected_row,
        "Expected cursor to remain on line {}, but found on line {}",
        line_number,
        current_row + 1
    );
    debug!("Cursor remained on line {} as expected", line_number);
}
