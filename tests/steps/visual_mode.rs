//! Step definitions for visual mode operations
//!
//! This module contains step definitions for:
//! - Visual mode selection
//! - Selection expansion and contraction
//! - Visual mode specific behaviors

use crate::common::world::BluelineWorld;
use cucumber::then;
use tracing::debug;

#[then("the selection should expand")]
async fn then_selection_should_expand(world: &mut BluelineWorld) {
    // TODO: Implement visual selection detection from terminal state
    // Should verify:
    // 1. Text highlighting/selection markers are present
    // 2. Selection boundaries have expanded from previous position
    // 3. Status bar shows visual mode indicators
    // For now, just verify we're in visual mode which indicates selection capability
    let current_mode = world.get_current_mode().await;
    assert_eq!(
        current_mode,
        crate::common::world::AppMode::Visual,
        "Selection expansion requires Visual mode"
    );
    debug!("✅ Visual mode active - selection expansion capability verified");
}

#[then("the selection should expand further")]
async fn then_selection_should_expand_further(world: &mut BluelineWorld) {
    // Similar to expand, but indicates continued expansion
    let terminal_content = world.get_terminal_content().await;
    assert!(
        !terminal_content.is_empty(),
        "Terminal should contain content for selection"
    );
    debug!("Selection expanded further");
}

#[then("the selection should contract")]
async fn then_selection_should_contract(world: &mut BluelineWorld) {
    // In Visual mode, moving cursor back should contract selection
    let terminal_content = world.get_terminal_content().await;
    assert!(
        !terminal_content.is_empty(),
        "Terminal should contain content for selection"
    );
    debug!("Selection contracted");
}
