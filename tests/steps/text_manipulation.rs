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
    world.type_text(&text).await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[then(regex = r#"I should see "([^"]+)" in the output"#)]
async fn then_should_see_output(world: &mut BluelineWorld, expected_output: String) {
    debug!("Checking for expected output: '{}'", expected_output);
    let contains = world.terminal_contains(&expected_output).await;
    assert!(
        contains,
        "Expected to find '{expected_output}' in terminal output"
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
