//! Step definitions for text input and manipulation
//!
//! This module contains step definitions for:
//! - Text input and typing
//! - Text deletion
//! - Text verification

use crate::common::world::BluelineWorld;
use cucumber::{then, when};
use tracing::{debug, info};

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
