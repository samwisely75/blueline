# Generic Cursor Movement Validation System

This document describes the new generic cursor movement validation system that was implemented to fix the testing gap that allowed cursor movement bugs to go undetected.

## The Problem

Previously, integration tests had a critical flaw where they only validated that cursor positions were "within reasonable bounds" rather than checking actual movement. This allowed bugs like missing ViewEvent emission to pass tests, causing cursor movement to appear to work while actually being broken.

### Specific Issues
- Tests only checked `cursor.column < 80` instead of verifying actual movement
- Mix of simulation vs VTE validation (inconsistent)
- No before/after position comparison
- Separate test functions for each movement type (not scalable)

## The Solution

### 1. Automatic Cursor Tracking

The `BluelineWorld` test harness now automatically captures cursor position before **every** key event:

```rust
pub async fn send_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) {
    // AUTOMATICALLY capture cursor position before EVERY key event
    let state = self.get_terminal_state().await;
    self.cursor_before_action = Some(state.cursor_position);
    self.cursor_history.push(CursorSnapshot {
        position: state.cursor_position,
        timestamp: std::time::Instant::now(),
        trigger: format!("{code:?}"),
    });
    
    // Send the actual key event...
}
```

### 2. Generic Step Definition

Instead of separate test steps for each movement type, we now have a single generic step that handles all cursor validation:

```rust
#[then(regex = r"^the cursor should (.+)$")]
async fn then_cursor_should(world: &mut BluelineWorld, expectation: String) {
    validate_cursor_movement(world, &expectation).await?;
}
```

### 3. Rich Expectation Parsing

The system supports natural language expectations:

```gherkin
# Relative movements
Then the cursor should move left 1 character
Then the cursor should move right 2 characters  
Then the cursor should move up 1 line
Then the cursor should move down 3 lines

# Absolute positions
Then the cursor should be at line 5 column 10

# Text anchors
Then the cursor should be at start of line
Then the cursor should be at end of line
Then the cursor should be at start of word
Then the cursor should be at end of word

# No movement
Then the cursor should not move
Then the cursor should remain at same position
```

### 4. VTE-Based Validation

All validation now uses the VTE parser to check actual terminal output, not simulation:

```rust
let before = world.get_cursor_before_action()
    .ok_or_else(|| anyhow!("No cursor position captured before action"))?;
let after = world.get_current_cursor_position().await;

// Validate actual movement occurred
validate_relative_movement(before, after, direction, amount)?;
```

## Benefits

### 1. Would Have Caught the ViewEvent Bug

The original bug where cursor movement commands didn't return ViewEvents would have been caught immediately:

- Command executes but doesn't emit ViewEvents → No render update
- VTE sees no cursor movement escape sequences  
- Test expects cursor at column N-1, but it's still at column N
- **Test fails with clear error message**

### 2. Scalable and Maintainable

- **One step definition** handles all cursor movement types
- **Easy to add new movement patterns** without changing core logic
- **Consistent validation** across all cursor tests
- **Clear error messages** with cursor history for debugging

### 3. Better Debugging Information

When tests fail, you get comprehensive information:

```
Cursor movement validation failed: Expected cursor to move left by 1 from column 10 to 9, but got 10
Last action: Char('h') at (10, 5)
Cursor history: [
    CursorSnapshot { position: (8, 5), trigger: "Char('l')" },
    CursorSnapshot { position: (9, 5), trigger: "Char('l')" },
    CursorSnapshot { position: (10, 5), trigger: "Char('h')" }
]
```

### 4. Future-Proof

New cursor movements or navigation commands automatically get validated without additional test code.

## Implementation Details

### Core Components

1. **CursorSnapshot** - Records position, timestamp, and trigger
2. **CursorExpectation** - Enum of all possible movement expectations
3. **Parsing Logic** - Converts natural language to expectations
4. **Validation Logic** - Checks actual movement against expectations

### File Structure

```
tests/
├── common/
│   ├── cursor_validation.rs  # Core validation logic
│   └── world.rs              # Automatic cursor tracking
├── steps/
│   └── navigation.rs         # Generic step definition
└── features/
    └── cursor_navigation.feature  # Test scenarios
```

### Usage in Feature Files

The generic step works with existing feature files without changes:

```gherkin
Scenario: Basic vim navigation (h,j,k,l)
  Given I am in Insert mode
  When I type "Line 1"
  And I press Escape
  And I press "h"
  Then the cursor should move left one character  # ← Works!
```

## Migration Guide

### For New Tests

Use the generic step pattern:

```gherkin
Then the cursor should move left 1 character
Then the cursor should be at line 2 column 5
Then the cursor should be at end of line
```

### For Existing Tests

The system is backward compatible. Existing specific step definitions still work, but new tests should use the generic pattern.

### Adding New Movement Types

To add support for new movement patterns:

1. Add to `CursorExpectation` enum if needed
2. Add parsing logic in `parse_cursor_expectation()`
3. Add validation logic if needed

Example:
```rust
// Add to parsing
if expectation.contains("next paragraph") {
    return Ok(CursorExpectation::TextPosition {
        anchor: TextAnchor::NextParagraph,
        text: None,
    });
}
```

## Testing the System

The generic validation system itself has comprehensive unit tests:

```bash
cargo test cursor_validation
```

Integration tests validate the complete flow:

```bash
cargo test --test integration_tests
```

## Conclusion

This generic cursor validation system eliminates the testing gap that allowed cursor movement bugs to slip through. It provides:

- **Automatic detection** of missing ViewEvent emission
- **Comprehensive validation** of actual cursor movement
- **Scalable test patterns** that don't require new code for each movement type
- **Better debugging** with detailed error messages and cursor history

The system ensures that any future cursor movement implementation bugs will be caught immediately during testing, not discovered later by users.