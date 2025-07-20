# REPL Architecture Refactoring - MVC Implementation

This document explains the new MVC-based architecture for the REPL module and how to test/use it.

## Architecture Overview

The original `repl.rs` has been refactored into a clean MVC architecture under the `src/repl/` directory:

```
src/repl/
├── mod.rs              # Module exports and overview
├── command.rs          # Command trait and interfaces  
├── commands/           # Specific command implementations
│   ├── mod.rs
│   ├── movement.rs     # Cursor movement commands (h, j, k, l, etc.)
│   └── editing.rs      # Text editing commands (i, A, Enter, etc.)
├── controller.rs       # Main controller and event loop
├── model.rs           # Data structures and application state
├── view.rs            # Rendering system and observers
└── new_repl.rs        # Integration entry point
```

## Design Principles

### Separation of Concerns

**Before (Monolithic)**:
- `Buffer` handled both data storage AND key processing
- `VimRepl` contained 1800+ lines mixing UI, logic, and state
- Key handlers directly modified multiple pieces of state

**After (MVC)**:
- **Model**: Pure data structures (`RequestBuffer`, `ResponseBuffer`, `AppState`)
- **View**: Rendering logic with observer pattern (`ViewManager`, pane renderers)
- **Controller**: Command orchestration and event dispatching (`ReplController`)

### Command Pattern

Each vim operation is now a separate `Command` implementation:

```rust
pub trait CommandV2 {
    fn process_detailed(&self, event: KeyEvent, state: &mut AppState) -> Result<CommandResult>;
    fn name(&self) -> &'static str;
}
```

Examples:
- `MoveCursorLeftCommand` - handles 'h' and Left arrow
- `InsertCharCommand` - handles character insertion in insert mode
- `SwitchPaneCommand` - handles Ctrl+W w pane switching

### Benefits

1. **Testability**: Each command can be unit tested in isolation
2. **Maintainability**: Changes to one command don't affect others
3. **Extensibility**: Adding new vim commands is just implementing the trait
4. **Performance**: Optimized rendering based on command results
5. **Debugging**: Clear command execution tracing

## Command Result System

Commands return detailed results to help the view layer optimize rendering:

```rust
pub struct CommandResult {
    pub handled: bool,           // Did this command process the event?
    pub content_changed: bool,   // Was buffer content modified?
    pub cursor_moved: bool,      // Did cursor position change?
    pub mode_changed: bool,      // Did editor mode change?
    pub pane_changed: bool,      // Did active pane change?
    pub scroll_occurred: bool,   // Did scrolling happen?
    pub status_message: Option<String>, // Status update
}
```

This enables the same three-tier rendering optimization as the original:
1. **Cursor-only updates** - fastest, just move cursor
2. **Content updates** - redraw affected pane only  
3. **Full updates** - complete screen redraw

## Testing the New Implementation

### Enable New REPL

Set the environment variable to use the new implementation:

```bash
export BLUELINE_NEW_REPL=1
cargo run -- --profile your_profile
```

### Current Status

**Working**:
- Basic cursor movement (h, j, k, l, arrow keys)
- Line start/end movement (0, $)
- Insert mode entry/exit (i, I, A, Esc)
- Character insertion and newlines
- Backspace deletion
- Pane switching (Ctrl+W w)
- Command mode entry (:)

**Not Yet Implemented**:
- HTTP request execution
- Response display
- Visual mode
- Copy/paste operations
- Advanced movement (w, b, gg, G)
- Scrolling commands
- Many other vim commands

## Development Workflow

### Adding New Commands

1. Create a new command struct in appropriate file under `commands/`
2. Implement the `CommandV2` trait
3. Register it in `ReplController::register_default_commands()`

Example:
```rust
pub struct WordForwardCommand;

impl CommandV2 for WordForwardCommand {
    fn process_detailed(&self, event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        if !matches!(event.code, KeyCode::Char('w')) { 
            return Ok(CommandResult::not_handled()); 
        }
        
        // Implement word movement logic...
        Ok(CommandResult::cursor_moved())
    }
    
    fn name(&self) -> &'static str { "WordForward" }
}
```

### Adding New View Components

1. Create a new observer implementing `RenderObserver`
2. Add it to `create_default_view_manager()`
3. Handle the specific rendering logic

### Testing Individual Commands

Commands can be unit tested in isolation:

```rust
#[test]
fn test_cursor_left_movement() {
    let mut state = AppState::new((80, 24), false);
    state.request_buffer.cursor_col = 5;
    
    let command = MoveCursorLeftCommand::new();
    let result = command.process_detailed(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), &mut state).unwrap();
    
    assert!(result.handled);
    assert!(result.cursor_moved);
    assert_eq!(state.request_buffer.cursor_col, 4);
}
```

## Migration Strategy

The refactoring maintains backward compatibility:

1. **Original REPL** remains in `old_repl.rs` (default behavior)
2. **New REPL** is opt-in via environment variable
3. **Gradual migration** of commands from old to new implementation
4. **Feature parity testing** before final switch

## Next Steps

1. **Implement missing commands** (word movement, gg/G, scrolling)
2. **Add HTTP request execution** (integrate with existing `HttpClient`)
3. **Implement response display** (populate and render `ResponseBuffer`)
4. **Add visual mode support** (selection and yank/delete operations)
5. **Performance testing** (ensure rendering optimizations work)
6. **Complete test coverage** (unit tests for all commands)

## Performance Considerations

The new architecture maintains the same rendering optimizations:

- **Minimal cursor updates** for pure navigation
- **Pane-specific updates** for content changes
- **Full screen refresh** only when necessary (mode changes, pane switches, etc.)

The command dispatch overhead is minimal since commands are tried in order until one handles the event, typically requiring only 1-3 command checks per keystroke.
