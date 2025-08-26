# Direct Command Architecture

## Overview

The Direct Command Architecture is a simplified command pattern that eliminates event-driven indirection. Commands execute their business logic directly on the ViewModel and Services, without emitting events for the controller to interpret.

## Problem with Event-Based Architecture

The original unified command architecture still required:
- Commands emit semantic events (ModelEvent)
- Controller interprets events in massive match statements
- Business logic remains scattered in the controller
- Controller grows to thousands of lines despite "simplification"

## Solution: Direct Execution

Commands now:
1. **Contain ALL business logic** - no delegation to controller
2. **Execute directly** on ViewModel and Services - no events
3. **Are self-contained** - everything needed is in the command

## Architecture Components

### DirectCommand Trait

```rust
pub trait DirectCommand: Send + Sync {
    /// Check if this command should handle the given key event
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool;
    
    /// Execute the command directly on ViewModel and Services
    fn execute(&self, view_model: &mut ViewModel, services: &mut Services) -> Result<()>;
    
    /// Get command name for debugging
    fn name(&self) -> &'static str;
}
```

### DirectCommandRegistry

Manages all direct commands and finds the first relevant command for any key event:

```rust
pub struct DirectCommandRegistry {
    commands: Vec<DirectCommandArc>,
}
```

### AppController Integration

The controller now acts as a simple dispatcher:

```rust
async fn handle_key_event_with_direct_first(&mut self, key_event: KeyEvent) -> Result<()> {
    // Try direct commands first
    if let Some(command) = self.direct_command_registry.find_command(key_event, mode, &context) {
        // Execute directly - no events!
        command.execute(&mut self.view_model, &mut self.services)?;
        self.render_if_needed()?;
        return Ok(());
    }
    
    // Fall back to old systems if needed
    self.handle_key_event_with_unified_first(key_event).await
}
```

## Implementation Example: Yank Command

```rust
impl DirectCommand for DirectYankCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if 'y' key in visual mode
        matches!(key_event.code, KeyCode::Char('y'))
            && matches!(mode, EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock)
            && context.has_selection
    }

    fn execute(&self, view_model: &mut ViewModel, services: &mut Services) -> Result<()> {
        // Get selected text
        let selected_text = view_model.get_selected_text()
            .ok_or_else(|| anyhow!("No selection"))?;
        
        // Store in yank service
        services.yank.yank(selected_text, yank_type)?;
        
        // Update UI state
        view_model.clear_visual_selection()?;
        view_model.set_mode(EditorMode::Normal);
        view_model.set_status_message("Text yanked");
        
        Ok(())
    }
}
```

## Benefits

1. **Simplicity** - No event indirection, commands do what they say
2. **Locality** - All logic for a feature in one place
3. **Testability** - Commands are isolated units
4. **Maintainability** - Easy to understand what each command does
5. **Controller Simplification** - Controller becomes a thin dispatcher

## Migration Strategy

### Phase 1: Infrastructure (Complete)
- ✅ Create DirectCommand trait
- ✅ Create DirectCommandRegistry
- ✅ Integrate with AppController

### Phase 2: Proof of Concept (Complete)
- ✅ Migrate YankSelectionCommand
- ✅ Migrate PasteAfterCommand
- ✅ Migrate PasteBeforeCommand

### Phase 3: Full Migration (In Progress)
- Migrate remaining commands in groups:
  - Visual mode commands (delete, cut, change)
  - Movement commands (hjkl, word navigation)
  - Mode change commands (i, a, v, etc.)
  - Ex commands (:q, :w, etc.)

### Phase 4: Cleanup
- Remove old CommandRegistry
- Remove CommandEvent enum
- Remove event processing methods
- Simplify AppController to ~200-300 lines

## Command Implementation Guidelines

1. **Self-Contained Logic**: Commands should not rely on controller methods
2. **Direct Manipulation**: Modify ViewModel and Services directly
3. **Error Handling**: Return Result<()> for proper error propagation
4. **Status Messages**: Set user feedback directly on ViewModel
5. **Logging**: Use tracing for debugging

## Testing Strategy

Commands are easily testable in isolation:

```rust
#[test]
fn test_yank_command() {
    let mut view_model = ViewModel::new();
    let mut services = Services::new();
    
    // Set up state
    view_model.set_mode(EditorMode::Visual);
    view_model.set_selection(/* ... */);
    
    // Execute command
    let command = DirectYankCommand::new();
    let result = command.execute(&mut view_model, &mut services);
    
    // Assert results
    assert!(result.is_ok());
    assert_eq!(view_model.get_mode(), EditorMode::Normal);
    assert!(services.yank.has_content());
}
```

## Future Improvements

1. **Command Composition**: Allow commands to call other commands
2. **Macro Generation**: Create macros for common command patterns
3. **Async Commands**: Support for async operations (HTTP, file I/O)
4. **Command History**: Track executed commands for undo/redo

## Conclusion

The Direct Command Architecture achieves the original goal of simplifying the AppController by moving ALL business logic into commands. This creates a more maintainable, testable, and understandable codebase.