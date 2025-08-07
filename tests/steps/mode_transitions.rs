//! Step definitions for mode transition tests

use crate::common::world::BluelineWorld;
use crossterm::event::{KeyCode, KeyModifiers};
use cucumber::{given, then, when};
use tracing::{debug, info};

// Background steps
#[given("the application is started with default settings")]
async fn app_started_with_default_settings(world: &mut BluelineWorld) {
    info!("=== Starting background step: app with default settings ===");
    info!("Step 1: Initializing world...");
    world.initialize().await;
    info!("Step 2: Starting app...");
    world.start_app(vec![]).await.expect("Failed to start app");
    info!("Step 3: App started, waiting for initialization...");

    // Give the app time to initialize
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    info!("Step 4: Background step complete");
}

// Given steps
#[given("I am in Insert mode")]
async fn given_insert_mode(world: &mut BluelineWorld) {
    debug!("Ensuring we are in Insert mode");
    // The application starts in Insert mode by default
    let _state = world.get_terminal_state().await;
    // TODO: Add mode detection from terminal state
    debug!("Current terminal state captured");
}

#[given("I am in Command mode")]
async fn given_command_mode(world: &mut BluelineWorld) {
    debug!("Ensuring we are in Command mode");
    // Press Escape to enter Command mode
    world
        .send_key_event(KeyCode::Esc, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

// When steps
#[when("the application starts")]
async fn when_application_starts(world: &mut BluelineWorld) {
    debug!("Application start event");
    // This is already handled by the background step
    world.tick().await.expect("Failed to tick");
}

#[when("I press Escape")]
async fn when_press_escape(world: &mut BluelineWorld) {
    info!("Pressing Escape key");
    world
        .send_key_event(KeyCode::Esc, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[when(regex = r#"I press "([^"]+)""#)]
async fn when_press_key(world: &mut BluelineWorld, key: String) {
    info!("Pressing key: {}", key);
    match key.as_str() {
        "i" => {
            world
                .send_key_event(KeyCode::Char('i'), KeyModifiers::empty())
                .await
        }
        _ => panic!("Unsupported key: {key}"),
    }
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[when("I press Enter")]
async fn when_press_enter(world: &mut BluelineWorld) {
    info!("Pressing Enter key");
    world.press_enter().await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[when(regex = r#"I type "([^"]+)""#)]
async fn when_type_text(world: &mut BluelineWorld, text: String) {
    info!("Typing text: {}", text);
    world.type_text(&text).await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

// Then steps
#[then("I should be in Insert mode")]
async fn then_should_be_insert_mode(world: &mut BluelineWorld) {
    debug!("Verifying Insert mode");
    let state = world.get_terminal_state().await;
    // TODO: Implement mode detection from terminal state
    // For now, we'll just capture the state for debugging
    state.debug_print();
}

#[then("I should be in Command mode")]
async fn then_should_be_command_mode(world: &mut BluelineWorld) {
    debug!("Verifying Command mode");
    let state = world.get_terminal_state().await;
    // TODO: Implement mode detection from terminal state
    state.debug_print();
}

#[then(regex = r#"the request pane should show line number "([^"]+)" in column (\d+)"#)]
async fn then_request_pane_line_number(
    world: &mut BluelineWorld,
    line_num: String,
    column: String,
) {
    info!(
        "Verifying request pane shows line number {} in column {}",
        line_num, column
    );
    let state = world.get_terminal_state().await;

    // Check if the first line contains the line number
    if let Some(first_line) = state.get_line(0) {
        debug!("First line content: '{}'", first_line);
        // The line number should appear at the specified column (0-indexed)
        let col: usize = column.parse().unwrap();
        if col > 0 && first_line.len() > col {
            let char_at_col = first_line.chars().nth(col - 1).unwrap_or(' ');
            assert_eq!(
                char_at_col,
                line_num.chars().next().unwrap_or('1'),
                "Expected line number '{line_num}' at column {column}, but found '{char_at_col}'"
            );
        }
    } else {
        panic!("No first line found in terminal state");
    }
}

#[then(regex = r#"the request pane should show "([^"]+)" for empty lines"#)]
async fn then_request_pane_empty_lines(world: &mut BluelineWorld, marker: String) {
    info!("Verifying request pane shows '{}' for empty lines", marker);
    let state = world.get_terminal_state().await;

    // Check that subsequent lines show the empty line marker
    for line_num in 1..5 {
        // Check first few lines
        if let Some(line) = state.get_line(line_num) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                assert!(
                    trimmed.starts_with(&marker),
                    "Expected empty line marker '{}' in line {}, but found '{}'",
                    marker,
                    line_num + 1,
                    trimmed
                );
            }
        }
    }
}

#[then(regex = r#"there should be a blinking block cursor at column (\d+)"#)]
async fn then_block_cursor_at_column(world: &mut BluelineWorld, column: String) {
    info!("Verifying block cursor at column {}", column);
    let state = world.get_terminal_state().await;
    let expected_col: u16 = column.parse::<u16>().unwrap() - 1; // Convert to 0-indexed

    // Check cursor position
    assert_eq!(
        state.cursor_position.0,
        expected_col,
        "Expected cursor at column {}, but found at column {}",
        expected_col + 1,
        state.cursor_position.0 + 1
    );
}

#[then(regex = r#"the status bar should show "([^"]+)" aligned to the right"#)]
async fn then_status_bar_shows(world: &mut BluelineWorld, expected_text: String) {
    info!("Verifying status bar shows: {}", expected_text);
    let state = world.get_terminal_state().await;

    // The status bar is typically at the bottom of the terminal
    let last_line_idx = (state.height as usize).saturating_sub(1);
    if let Some(status_line) = state.get_line(last_line_idx) {
        debug!("Status line content: '{}'", status_line);
        assert!(
            status_line.contains(&expected_text),
            "Expected status bar to contain '{expected_text}', but found '{status_line}'"
        );
    } else {
        panic!("No status line found");
    }
}

#[then("there should be no response pane visible")]
async fn then_no_response_pane(world: &mut BluelineWorld) {
    info!("Verifying no response pane is visible");
    let state = world.get_terminal_state().await;

    // TODO: Implement response pane detection logic
    // For now, we'll just capture the state for analysis
    debug!("Terminal state captured for response pane verification");
    state.debug_print();
}

#[then("the cursor should change appearance")]
async fn then_cursor_changes_appearance(_world: &mut BluelineWorld) {
    info!("Verifying cursor appearance change");
    // TODO: Implement cursor appearance detection
    // This is complex to detect from terminal output alone
    debug!("Cursor appearance change verification - placeholder");
}

#[then(regex = r#"I should see "([^"]+)" in the output"#)]
async fn then_should_see_output(world: &mut BluelineWorld, expected_output: String) {
    info!("Verifying output contains: {}", expected_output);
    let contains = world.terminal_contains(&expected_output).await;
    assert!(
        contains,
        "Expected to find '{expected_output}' in terminal output"
    );
}

#[then("I should remain in Insert mode")]
async fn then_should_remain_insert_mode(world: &mut BluelineWorld) {
    debug!("Verifying still in Insert mode");
    // Same as checking for Insert mode
    then_should_be_insert_mode(world).await;
}
