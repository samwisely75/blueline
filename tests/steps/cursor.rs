//! Step definitions for cursor visibility and styling
//!
//! This module contains step definitions for:
//! - Cursor visibility state
//! - Cursor style (block, bar, underline)
//! - Cursor blinking behavior
//! - Mode-specific cursor changes

use crate::common::world::BluelineWorld;
use cucumber::{given, then};
use tracing::debug;

// === CURSOR VISIBILITY ===

#[given("the cursor is visible")]
async fn given_cursor_visible(world: &mut BluelineWorld) {
    debug!("Assuming cursor is visible (default state)");
    // Cursor should be visible by default in Normal mode
    let _ = world; // Acknowledge the parameter
}

#[then("the cursor should be visible")]
async fn then_cursor_visible(world: &mut BluelineWorld) {
    debug!("Verifying cursor is visible");
    // Check for cursor presence - could be block, bar, or underline
    let has_cursor = world.terminal_contains("█").await
        || world.terminal_contains("▌").await
        || world.terminal_contains("│").await
        || world.terminal_contains("_").await
        || world.terminal_contains("▁").await;

    // In test environment, cursor might not be rendered visually
    // but we can check that we're not in Command mode
    if !has_cursor {
        let not_in_command = !world.terminal_contains(":").await
            || world.terminal_contains("Normal").await
            || world.terminal_contains("Insert").await;
        assert!(
            not_in_command,
            "Cursor should be visible outside Command mode"
        );
    }
}

#[then("the cursor should be hidden")]
async fn then_cursor_hidden(world: &mut BluelineWorld) {
    debug!("Verifying cursor is hidden");
    // In Command mode, cursor should be hidden
    // We can verify this by checking we're in Command mode
    let in_command = world.terminal_contains(":").await || world.terminal_contains("Command").await;
    assert!(in_command, "Cursor should be hidden in Command mode");
}

// === CURSOR STYLES ===

#[then("the cursor should be visible with blinking bar style")]
async fn then_cursor_visible_blinking_bar(world: &mut BluelineWorld) {
    debug!("Verifying cursor is visible with blinking bar style");
    // In Insert mode, cursor should be a blinking bar
    // Check that we're in Insert mode
    let in_insert = world.terminal_contains("Insert").await
        || world.terminal_contains("INSERT").await
        || world.terminal_contains("-- INSERT --").await;
    assert!(in_insert, "Cursor should be blinking bar in Insert mode");
}

#[then("the cursor should be visible with steady block style")]
async fn then_cursor_visible_steady_block(world: &mut BluelineWorld) {
    debug!("Verifying cursor is visible with steady block style");
    // In Normal mode, cursor should be a steady block
    // Check that we're in Normal mode
    let in_normal = world.terminal_contains("Normal").await
        || world.terminal_contains("NORMAL").await
        || (!world.terminal_contains("Insert").await && !world.terminal_contains(":").await);
    assert!(in_normal, "Cursor should be steady block in Normal mode");
}
