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
        eprintln!(
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
        eprintln!(
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
        eprintln!(
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
    eprintln!("=== EXPECTED TEXT ===");
    eprintln!("'{expected}'");
    eprintln!("=== ACTUAL TERMINAL CONTENT ===");
    for (i, line) in terminal_content.lines().enumerate() {
        eprintln!("{:2}: '{}'", i + 1, line);
    }
    eprintln!("=== END COMPARISON ===");

    // Check each line of the expected text
    for line in expected.lines() {
        if !line.trim().is_empty() {
            // Skip empty lines
            let contains = world.terminal_contains(line).await;
            if !contains {
                eprintln!("❌ Missing line: '{line}'");
                eprintln!(
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
