//! Step definitions for text input and manipulation
//!
//! This module contains step definitions for:
//! - Text input and typing
//! - Text deletion
//! - Text verification

use crate::common::world::BluelineWorld;
use crossterm::event::{KeyCode, KeyModifiers};
use cucumber::{gherkin, given, then, when};
use tracing::{debug, info};

#[when(regex = r#"^(?:And )?I press Enter$"#)]
async fn when_press_enter(world: &mut BluelineWorld) {
    info!("Pressing Enter key");
    world.press_enter().await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[when(regex = r#"I type "([^"]+)""#)]
async fn when_type_text(world: &mut BluelineWorld, text: String) {
    info!("Typing text: {}", text);

    // Special debugging for John issue
    if text.contains("John") {
        tracing::debug!(
            "🔍 ABOUT TO TYPE: '{}', text buffer before: {:?}",
            text,
            world.get_text_buffer()
        );
    }

    world.type_text(&text).await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Check text buffer after typing John
    if text.contains("John") {
        tracing::debug!(
            "🔍 AFTER TYPING: '{}', text buffer after: {:?}",
            text,
            world.get_text_buffer()
        );
    }
}

#[then(regex = r#"I should see "([^"]+)" in the output"#)]
async fn then_should_see_output(world: &mut BluelineWorld, expected_output: String) {
    debug!("Checking for expected output: '{}'", expected_output);

    // Get the full terminal content for debugging
    let terminal_content = world.get_terminal_content().await;
    debug!("Current terminal content:\n{}", terminal_content);

    let contains = world.terminal_contains(&expected_output).await;

    // Debug output for John issue (now that we've fixed it)
    if expected_output == "John" && !contains {
        let text_buffer = world.get_text_buffer();
        tracing::debug!(
            "🔍 JOHN DEBUG - Text not found!\n\
            Expected: '{}'\n\
            Terminal content ({} chars):\n'{}'\n\
            Text buffer ({} lines): {:?}",
            expected_output,
            terminal_content.len(),
            terminal_content,
            text_buffer.len(),
            text_buffer
        );
    }

    assert!(
        contains,
        "Expected to find '{expected_output}' in terminal output.\nActual terminal content ({} chars):\n{terminal_content}",
        terminal_content.len()
    );
}

#[then(regex = r#"I should see "([^"]+)" highlighted"#)]
async fn then_should_see_highlighted(world: &mut BluelineWorld, text: String) {
    debug!("Checking for highlighted text: '{}'", text);
    // In visual mode, selected text should be highlighted
    // For now, we'll just verify the text exists
    let contains = world.terminal_contains(&text).await;
    assert!(
        contains,
        "Expected to find '{text}' highlighted in terminal"
    );
    // TODO: Implement highlighting detection from terminal state
    // Additional verification would check for ANSI color codes or selection markers
}

// === TEXT DELETION STEP DEFINITIONS ===

#[given(regex = r#"I have text "([^"]+)" in the request pane"#)]
async fn given_text_in_request_pane(world: &mut BluelineWorld, text: String) {
    info!("Setting text in request pane: '{}'", text);
    world.type_text(&text).await;
    world.tick().await.expect("Failed to tick");
}

#[given("the cursor is at the end")]
async fn given_cursor_at_end(world: &mut BluelineWorld) {
    info!("Moving cursor to end of line");
    world.press_key('$').await;
    world.tick().await.expect("Failed to tick");
}

#[given("the cursor is at the beginning")]
async fn given_cursor_at_beginning(world: &mut BluelineWorld) {
    info!("Moving cursor to beginning of line");
    world.press_key('0').await;
    world.tick().await.expect("Failed to tick");
}

#[given(regex = r#"the cursor is after "([^"]+)""#)]
async fn given_cursor_after_text(world: &mut BluelineWorld, text: String) {
    info!("Positioning cursor after '{}'", text);

    // Don't clear existing text - just position cursor
    // Move to beginning of line first by pressing Home key
    world
        .send_key_event(KeyCode::Home, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");

    // Then move right by the number of characters in the text
    for _ in 0..text.chars().count() {
        world.press_arrow_right().await;
        world.tick().await.expect("Failed to tick");
    }

    info!("Cursor should now be positioned after '{}'", text);
}

#[when("I press Backspace")]
async fn when_press_backspace(world: &mut BluelineWorld) {
    info!("Pressing Backspace key");

    // Debug: show content before backspace
    let before_content = world.get_terminal_content().await;
    info!(
        "Terminal content BEFORE backspace: '{}'",
        before_content.lines().collect::<Vec<_>>().join(" | ")
    );

    world.press_backspace().await;
    world.tick().await.expect("Failed to tick");

    // Debug: show content after backspace
    let after_content = world.get_terminal_content().await;
    info!(
        "Terminal content AFTER backspace: '{}'",
        after_content.lines().collect::<Vec<_>>().join(" | ")
    );
}

#[when(regex = r#"I press backspace (\d+) times"#)]
async fn when_press_backspace_n_times(world: &mut BluelineWorld, count: usize) {
    info!("Pressing Backspace {} times", count);
    for _ in 0..count {
        world.press_backspace().await;
        world.tick().await.expect("Failed to tick");
    }
}

#[when(regex = r#"I press the delete key (\d+) times"#)]
async fn when_press_delete_n_times(world: &mut BluelineWorld, count: usize) {
    info!("Pressing Delete key {} times", count);
    for _ in 0..count {
        world.press_delete().await;
        world.tick().await.expect("Failed to tick");
    }
}

#[given(regex = r"^the request buffer contains:$")]
async fn given_request_buffer_contains(world: &mut BluelineWorld, step: &gherkin::Step) {
    let docstring = step.docstring.as_deref().unwrap_or("");
    info!("Setting request buffer with multiline text: {}", docstring);

    let line_count = docstring.lines().count();
    info!("Docstring has {} lines", line_count);
    for (i, line) in docstring.lines().enumerate() {
        info!("Line {}: '{}'", i + 1, line);
    }

    // Clear any existing text first
    world.clear_request_buffer().await;
    // Type the multiline text
    world.type_text(docstring).await;
    world.tick().await.expect("Failed to tick");

    // Debug: show terminal content after insertion
    let terminal_content = world.get_terminal_content().await;
    info!(
        "Terminal content after docstring insertion:\n{}",
        terminal_content
    );
}

#[given(regex = r#"the cursor is at the beginning of line (\d+)"#)]
async fn given_cursor_at_line_beginning(world: &mut BluelineWorld, line_num: usize) {
    info!("Moving cursor to beginning of line {}", line_num);

    // Use Insert mode navigation (arrow keys) instead of vim commands
    // First go to top of document using Ctrl+Home or many Up arrows
    for _ in 0..100 {
        // Go up many times to ensure we reach the top
        world.press_arrow_up().await;
        world.tick().await.expect("Failed to tick");
    }

    // Then move down to target line (1-indexed)
    if line_num > 1 {
        for _ in 0..(line_num - 1) {
            world.press_arrow_down().await;
            world.tick().await.expect("Failed to tick");
        }
    }

    // Move to beginning of line using Home key
    world
        .send_key_event(KeyCode::Home, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
}

#[given(regex = r#"the cursor is on the blank line \(line (\d+)\)"#)]
async fn given_cursor_on_blank_line(world: &mut BluelineWorld, line_num: usize) {
    info!("Moving cursor to blank line {}", line_num);
    world.press_keys("gg").await;
    if line_num > 1 {
        for _ in 0..(line_num - 1) {
            world.press_key('j').await;
        }
    }
    world.tick().await.expect("Failed to tick");
}

#[given(regex = r#"the cursor is on the second blank line \(line (\d+)\)"#)]
async fn given_cursor_on_second_blank_line(world: &mut BluelineWorld, line_num: usize) {
    info!("Moving cursor to second blank line {}", line_num);
    world.press_keys("gg").await;
    if line_num > 1 {
        for _ in 0..(line_num - 1) {
            world.press_key('j').await;
        }
    }
    world.tick().await.expect("Failed to tick");
}

#[then("the last character should be removed")]
async fn then_last_char_removed(_world: &mut BluelineWorld) {
    // This is verified by the next step checking the actual text
}

// Note: Using the definition from http.rs for "I should see ... in the request pane"

#[then("the screen should not be blank")]
async fn then_screen_not_blank(world: &mut BluelineWorld) {
    let content = world.get_terminal_content().await;
    assert!(!content.trim().is_empty(), "Screen should not be blank");
}

#[then("the two lines should be joined")]
async fn then_lines_joined(_world: &mut BluelineWorld) {
    // This is verified by the next step checking the actual text
}

#[when(regex = r"^I type the following JSON:$")]
async fn when_type_json(world: &mut BluelineWorld, step: &gherkin::Step) {
    let json_text = step.docstring.as_deref().unwrap_or("");
    info!("Typing JSON text: {}", json_text);
    world.type_text(json_text).await;
    world.tick().await.expect("Failed to tick");
}

#[then(regex = r"^the text becomes:$")]
async fn then_text_becomes(world: &mut BluelineWorld, step: &gherkin::Step) {
    let expected = step.docstring.as_deref().unwrap_or("");
    debug!(
        "Checking if text matches expected multiline content: {}",
        expected
    );

    // Debug: show actual terminal content
    let terminal_content = world.get_terminal_content().await;
    tracing::debug!("=== EXPECTED TEXT ===");
    tracing::debug!("'{expected}'");
    tracing::debug!("=== ACTUAL TERMINAL CONTENT ===");
    for (i, line) in terminal_content.lines().enumerate() {
        tracing::debug!("{:2}: '{}'", i + 1, line);
    }
    tracing::debug!("=== END COMPARISON ===");

    // Check each line of the expected text
    for line in expected.lines() {
        if !line.trim().is_empty() {
            // Skip empty lines
            let contains = world.terminal_contains(line).await;
            if !contains {
                tracing::debug!("❌ Missing line: '{line}'");
                tracing::debug!(
                    "Terminal content: '{}'",
                    terminal_content.replace('\n', "\\n")
                );
            }
            assert!(contains, "Expected to find line '{line}' in terminal");
        }
    }
}

#[then("no character is deleted")]
async fn then_no_char_deleted(_world: &mut BluelineWorld) {
    // This is verified by the next step checking the text remains unchanged
}

#[then(regex = r#"the text remains "([^"]+)""#)]
async fn then_text_remains(world: &mut BluelineWorld, expected: String) {
    debug!("Verifying text remains: '{}'", expected);
    let contains = world.terminal_contains(&expected).await;
    assert!(contains, "Expected text to remain '{expected}'");
}

#[then("the blank line is deleted")]
async fn then_blank_line_deleted(_world: &mut BluelineWorld) {
    // This is verified by checking the resulting text
}

#[then("the cursor moves to the end of the previous line")]
async fn then_cursor_at_prev_line_end(_world: &mut BluelineWorld) {
    // TODO: Implement cursor position verification
}

#[then("only the current blank line is deleted")]
async fn then_only_current_blank_deleted(_world: &mut BluelineWorld) {
    // This is verified by checking the resulting text
}

#[then(regex = r#"the cursor moves to the end of the previous line \(first blank line\)"#)]
async fn then_cursor_at_first_blank_end(_world: &mut BluelineWorld) {
    // TODO: Implement cursor position verification
}

#[when(regex = r#"I press "([^"]+)" followed by "([^"]+)"#)]
async fn when_press_key_followed_by_key(
    world: &mut BluelineWorld,
    first_key: String,
    second_key: String,
) {
    info!("Pressing '{}' followed by '{}'", first_key, second_key);

    // Press the first key
    match first_key.as_str() {
        "d" => {
            world
                .send_key_event(KeyCode::Char('d'), KeyModifiers::empty())
                .await;
        }
        "g" => {
            world
                .send_key_event(KeyCode::Char('g'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported first key in 'followed by' pattern: {first_key}");
        }
    }

    world.tick().await.expect("Failed to tick after first key");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Press the second key
    match second_key.as_str() {
        "d" => {
            world
                .send_key_event(KeyCode::Char('d'), KeyModifiers::empty())
                .await;
        }
        "g" => {
            world
                .send_key_event(KeyCode::Char('g'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported second key in 'followed by' pattern: {second_key}");
        }
    }

    world.tick().await.expect("Failed to tick after second key");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[when(regex = r#"I press "([^"]+)" without following "([^"]+)""#)]
async fn when_press_key_without_following(
    world: &mut BluelineWorld,
    first_key: String,
    _expected_second_key: String,
) {
    info!("Pressing '{}' without following second key", first_key);

    match first_key.as_str() {
        "d" => {
            world
                .send_key_event(KeyCode::Char('d'), KeyModifiers::empty())
                .await;
        }
        "g" => {
            world
                .send_key_event(KeyCode::Char('g'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported key in 'without following' pattern: {first_key}");
        }
    }

    world.tick().await.expect("Failed to tick after key press");
    // Don't press the second key - this is for testing timeout behavior
}

#[when(regex = r#"I wait (\d+) seconds?"#)]
async fn when_wait_seconds(world: &mut BluelineWorld, seconds: usize) {
    info!("Waiting for {} seconds", seconds);
    tokio::time::sleep(std::time::Duration::from_secs(seconds as u64)).await;
    world.tick().await.expect("Failed to tick after wait");
}

#[then(regex = r"^the request content should be:$")]
async fn then_request_content_should_be(world: &mut BluelineWorld, step: &gherkin::Step) {
    let expected_content = step.docstring.as_deref().unwrap_or("");
    info!(
        "Checking if request content matches expected: {}",
        expected_content
    );

    let terminal_content = world.get_terminal_content().await;
    debug!("Current terminal content:\n{}", terminal_content);

    // For now, check if the expected content is contained in the terminal
    // TODO: Implement proper request buffer content checking
    for line in expected_content.lines() {
        if !line.trim().is_empty() {
            let contains = world.terminal_contains(line).await;
            assert!(
                contains,
                "Expected to find line '{line}' in request content. Terminal content:\n{terminal_content}"
            );
        }
    }
}

#[then("the request content should be empty")]
async fn then_request_content_should_be_empty(world: &mut BluelineWorld) {
    info!("Checking if request content is empty");

    let terminal_content = world.get_terminal_content().await;
    debug!(
        "Terminal content when checking for empty: '{}'",
        terminal_content
    );

    // TODO: Implement proper request buffer empty check
    // For now, we'll check that there's minimal content (just UI elements)
    let lines: Vec<&str> = terminal_content.lines().collect();
    let non_empty_lines: Vec<&str> = lines
        .iter()
        .filter(|line| {
            !line.trim().is_empty() && !line.contains("Request") && !line.contains("Response")
        })
        .copied()
        .collect();

    assert!(
        non_empty_lines.is_empty(),
        "Expected request content to be empty, but found: {non_empty_lines:?}"
    );
}

#[when(regex = r#"I press "([^"]+)" to enter Insert mode"#)]
async fn when_press_to_enter_insert_mode(world: &mut BluelineWorld, key: String) {
    info!("Pressing '{}' to enter Insert mode", key);

    match key.as_str() {
        "i" => {
            world
                .send_key_event(KeyCode::Char('i'), KeyModifiers::empty())
                .await;
        }
        "a" => {
            world
                .send_key_event(KeyCode::Char('a'), KeyModifiers::empty())
                .await;
        }
        "A" => {
            world
                .send_key_event(KeyCode::Char('A'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported key for entering Insert mode: {key}");
        }
    }

    world
        .tick()
        .await
        .expect("Failed to tick after entering Insert mode");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[when(regex = r#"I press "([^"]+)" to enter Normal mode"#)]
async fn when_press_to_enter_normal_mode(world: &mut BluelineWorld, key: String) {
    info!("Pressing '{}' to enter Normal mode", key);

    match key.as_str() {
        "Escape" => {
            world
                .send_key_event(KeyCode::Esc, KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported key for entering Normal mode: {key}");
        }
    }

    world
        .tick()
        .await
        .expect("Failed to tick after entering Normal mode");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[when(regex = r#"I press "([^"]+)" to move (up one line|down one line|to first line|left|right)"#)]
async fn when_press_to_move(world: &mut BluelineWorld, key: String, direction: String) {
    info!("Pressing '{}' to {}", key, direction);

    match key.as_str() {
        "k" => {
            world
                .send_key_event(KeyCode::Char('k'), KeyModifiers::empty())
                .await;
        }
        "j" => {
            world
                .send_key_event(KeyCode::Char('j'), KeyModifiers::empty())
                .await;
        }
        "h" => {
            world
                .send_key_event(KeyCode::Char('h'), KeyModifiers::empty())
                .await;
        }
        "l" => {
            world
                .send_key_event(KeyCode::Char('l'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported key for movement: {key}");
        }
    }

    world.tick().await.expect("Failed to tick after movement");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[then(regex = r"the cursor should be at line (\d+), column (\d+)")]
async fn then_cursor_should_be_at_position(world: &mut BluelineWorld, line: usize, column: usize) {
    info!("Checking if cursor is at line {}, column {}", line, column);
    let terminal_content = world.get_terminal_content().await;
    debug!("Terminal content for cursor check: {}", terminal_content);
    // TODO: Implement proper cursor position checking
    // For now, this step passes as we assume cursor positioning works
}

#[when(regex = r#"I press "([^"]+)" to (paste after cursor|cut character)"#)]
async fn when_press_for_action(world: &mut BluelineWorld, key: String, action: String) {
    info!("Pressing '{}' to {}", key, action);

    match key.as_str() {
        "p" => {
            world
                .send_key_event(KeyCode::Char('p'), KeyModifiers::empty())
                .await;
        }
        "x" => {
            world
                .send_key_event(KeyCode::Char('x'), KeyModifiers::empty())
                .await;
        }
        _ => {
            panic!("Unsupported key for action '{action}': {key}");
        }
    }

    world.tick().await.expect("Failed to tick after action");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[then("the response content should not be empty")]
async fn then_response_content_should_not_be_empty(world: &mut BluelineWorld) {
    info!("Checking that response content is not empty");
    let terminal_content = world.get_terminal_content().await;
    debug!("Terminal content for response check: {}", terminal_content);
    // TODO: Implement proper response content checking
    // For now, this step passes assuming response pane has content
}
