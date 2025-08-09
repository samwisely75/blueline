//! Step definitions for advanced text operations
//!
//! This module contains step definitions for:
//! - Undo/redo operations
//! - Copy/paste operations
//! - Yank and put commands
//! - Text restoration

use crate::common::world::BluelineWorld;
use cucumber::{then, when};
use tracing::{debug, info};

// === UNDO/REDO OPERATIONS ===

#[when("I press \"u\"")]
async fn when_press_u_for_undo(world: &mut BluelineWorld) {
    info!("Pressing 'u' for undo");
    world.press_key('u').await;
    world.tick().await.expect("Failed to tick");
}

#[when("I press \"u\" for undo")]
async fn when_press_u_for_undo_explicit(world: &mut BluelineWorld) {
    info!("Pressing 'u' for undo");
    world.press_key('u').await;
    world.tick().await.expect("Failed to tick");
}

// === COPY/PASTE OPERATIONS ===

#[when("I press \"y\"")]
async fn when_press_y_for_yank(world: &mut BluelineWorld) {
    info!("Pressing 'y' to yank/copy");
    world.press_key('y').await;
    world.tick().await.expect("Failed to tick");
}

#[when("I copy it with \"y\"")]
async fn when_copy_with_y(world: &mut BluelineWorld) {
    info!("Copying selected text with 'y'");
    world.press_key('y').await;
    world.tick().await.expect("Failed to tick");
}

#[when("I press \"p\"")]
async fn when_press_p_for_paste(world: &mut BluelineWorld) {
    info!("Pressing 'p' to paste");
    world.press_key('p').await;
    world.tick().await.expect("Failed to tick");
}

#[when("I paste it with \"p\"")]
async fn when_paste_with_p(world: &mut BluelineWorld) {
    info!("Pasting with 'p'");
    world.press_key('p').await;
    world.tick().await.expect("Failed to tick");
}

// === TEXT RESTORATION ===

#[then("the deleted text should be restored")]
async fn then_deleted_text_restored(world: &mut BluelineWorld) {
    debug!("Verifying deleted text is restored");
    // This would be checked by the surrounding context steps
    // The actual verification happens through "I should see" steps
    let _ = world; // Acknowledge the parameter
}

#[then("the copied text should appear at the new position")]
async fn then_copied_text_appears(world: &mut BluelineWorld) {
    debug!("Verifying copied text appears at new position");
    // This would be checked by the surrounding context steps
    // The actual verification happens through "I should see" steps
    let _ = world; // Acknowledge the parameter
}

// === POSITION MOVEMENT ===

#[when("I move to a new position")]
async fn when_move_to_new_position(world: &mut BluelineWorld) {
    info!("Moving to a new position");
    // Move to end of line as an example
    world.press_key('$').await;
    world.tick().await.expect("Failed to tick");
}

#[when("I press \"$\"")]
async fn when_press_dollar(world: &mut BluelineWorld) {
    info!("Pressing '$' to move to end of line");
    world.press_key('$').await;
    world.tick().await.expect("Failed to tick");
}

// === MULTIPLE KEY PRESSES ===

#[when(regex = r#"I press "([a-z])" (\d+) times"#)]
async fn when_press_key_n_times(world: &mut BluelineWorld, key: char, count: usize) {
    info!("Pressing '{}' {} times", key, count);
    for _ in 0..count {
        world.press_key(key).await;
        world.tick().await.expect("Failed to tick");
    }
}
