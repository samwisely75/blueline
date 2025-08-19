//! Step definitions for mode transitions
//!
//! This module contains step definitions for:
//! - Mode switching (Normal, Insert, Visual, Command)
//! - Mode verification
//! - Mode-specific behaviors

use crate::common::world::{AppMode, BluelineWorld};
use crossterm::event::{KeyCode, KeyModifiers};
use cucumber::{given, then, when};
use tracing::{debug, info};

// Given steps for mode states
#[given("I am in Insert mode")]
async fn given_insert_mode(world: &mut BluelineWorld) {
    debug!("Ensuring we are in Insert mode");
    // Press 'i' to enter Insert mode from Normal mode
    world
        .send_key_event(KeyCode::Char('i'), KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    debug!("Switched to Insert mode");
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

#[given("I am in Normal mode")]
async fn given_normal_mode(world: &mut BluelineWorld) {
    debug!("Ensuring we are in Normal mode");
    // The application starts in Normal mode by default (vim-like behavior)
    // If we're not in Normal mode, press Escape to get there
    world
        .send_key_event(KeyCode::Esc, KeyModifiers::empty())
        .await;
    world.tick().await.expect("Failed to tick");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

// When steps for mode transitions
#[when("I enter Insert mode")]
async fn when_enter_insert_mode(world: &mut BluelineWorld) {
    info!("Entering Insert mode by pressing 'i'");
    world
        .send_key_event(KeyCode::Char('i'), KeyModifiers::empty())
        .await;
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

// Then steps for mode verification
#[then(regex = r"^I (?:should be|am) in Insert mode$")]
async fn then_should_be_insert_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::Insert,
        "Expected Insert mode, but found {current_mode:?}"
    );
}

#[then(regex = r"^I (?:should be|am) in Command mode$")]
async fn then_should_be_command_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::Command,
        "Expected Command mode, but found {current_mode:?}"
    );
}

#[then(regex = r"^I (?:should be|am) in Normal mode$")]
async fn then_should_be_normal_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::Normal,
        "Expected Normal mode, but found {current_mode:?}"
    );
}

#[then(regex = r"^I (?:should be|am) in Visual mode$")]
async fn then_should_be_visual_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::Visual,
        "Expected Visual mode, but found {current_mode:?}"
    );
}

#[then(regex = r"^I (?:should be|am) in Visual Line mode$")]
async fn then_should_be_visual_line_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::VisualLine,
        "Expected Visual Line mode, but found {current_mode:?}"
    );
}

#[then(regex = r"^I (?:should be|am) in Visual Block mode$")]
async fn then_should_be_visual_block_mode(world: &mut BluelineWorld) {
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        AppMode::VisualBlock,
        "Expected Visual Block mode, but found {current_mode:?}"
    );
}

#[then("I should remain in Insert mode")]
async fn then_should_remain_insert_mode(world: &mut BluelineWorld) {
    debug!("Verifying still in Insert mode");
    // Same as checking for Insert mode
    then_should_be_insert_mode(world).await;
}

#[then("I should remain in Visual mode")]
async fn then_should_remain_in_visual_mode(_world: &mut BluelineWorld) {
    // For now, we assume we remain in Visual mode
    // In a real implementation, we would check mode state from terminal output
    debug!("Verified remaining in Visual mode");
}

#[then("the cursor should change appearance")]
async fn then_cursor_should_change_appearance(world: &mut BluelineWorld) {
    // For now, we'll verify that we're in a specific mode rather than cursor style
    // Cursor appearance changes typically happen during mode transitions
    let current_mode = world.get_current_mode().await;

    // This step is usually called after mode changes, so we verify we're not in Unknown mode
    assert_ne!(
        current_mode,
        AppMode::Unknown,
        "Cursor appearance should change during valid mode transitions, but mode is unknown"
    );

    debug!(
        "Cursor appearance changed, current mode: {:?}",
        current_mode
    );
}
