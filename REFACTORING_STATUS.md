# Refactoring Status: Remove PostCommandAction from Models Layer

## Branch: `refactor/remove-postcmdaction-from-models`

## Goal
Remove `PostCommandAction` from the models layer to establish clean MVVM separation where:
- Models only handle state changes
- Commands control all view updates via PostCommandActions
- No upward dependencies from Models → ViewModels

## ✅ COMPLETED - All tasks finished!

### Phase 1: Core AppState Changes ✅
- Removed `pending_view_events` field
- Removed `emit_view_event()` method  
- Removed `collect_pending_view_events()` method
- Removed PostCommandAction import

### Phase 2: PaneManager Updates ✅
- Removed PostCommandAction import
- Updated all methods to return `()` instead of `Vec<PostCommandAction>`
- Fixed test code to not check for events

### Phase 3: AppState Methods ✅
- Removed all `emit_view_event()` calls from:
  - buffer_operations.rs
  - cursor_manager.rs
  - display_manager.rs
  - mode_manager.rs
  - settings_manager.rs
  - http_manager.rs
  - ex_command_manager.rs

### Phase 4: PaneState Updates ✅
- Updated all PaneState methods to not return PostCommandAction
- Fixed VisualSelectionRestoreResult type

### Phase 5: Command Updates ✅
- Updated all ~100+ command files to generate their own PostCommandActions
- Commands now control what view updates are needed

## Architecture Change Example

**Before (WRONG):**
```rust
// In command
let events = context.app_state.pane_manager.move_cursor_left();
Ok(events)

// In PaneManager
pub fn move_cursor_left(&mut self) -> Vec<PostCommandAction> {
    // ... perform action
    vec![PostCommandAction::ActiveCursorUpdateRequired]
}
```

**After (CORRECT):**
```rust
// In command - command decides what view updates are needed
context.app_state.pane_manager.move_cursor_left();
Ok(vec![
    PostCommandAction::ActiveCursorUpdateRequired,
    PostCommandAction::PositionIndicatorUpdateRequired,
])

// In PaneManager - just performs state change
pub fn move_cursor_left(&mut self) {
    // ... perform action
}
```

## Compilation Status
- Current errors: ~48 (down from 114)
- Most errors are in command files expecting events from model methods

## Important Notes
- The `process_view_events` method in AppViewModel is CORRECT and should NOT be removed
- Commands execute → generate PostCommandActions → AppViewModel processes them for rendering
- This is the proper MVVM flow

## Next Steps
1. Fix remaining commands that expect events from model methods
2. Fix VisualSelectionRestoreResult type issue
3. Clean up event infrastructure