# MVVM/Event-Driven Architecture Transition Strategy

## Overview

This document outlines a phased approach to transition the Blueline codebase from MVC to MVVM with event-driven architecture, addressing the current issue where Commands are too concerned with view logic.

## Target Architecture

### Core Principles
1. **Pure Model**: Contains only logical state (content, cursor position, mode)
2. **ViewModel**: Bridges Model and View, manages display state
3. **Event-Driven**: Components communicate through events, not direct coupling
4. **Command Simplification**: Commands only perform state transitions

### Architecture Diagram
```
┌─────────────┐     Events      ┌──────────────┐     Display Events    ┌────────────┐
│   Commands  │ ──────────────> │    Model     │ ───────────────────> │ ViewModel  │
│  (Stateless)│                  │ (Pure State) │                      │  (Display) │
└─────────────┘                  └──────────────┘                      └────────────┘
       ↑                                                                       │
       │                                                                       ↓
       │                         ┌──────────────┐     Render Events    ┌────────────┐
       └──────── Input Events ── │  Controller  │ <─────────────────── │    View    │
                                 │   (Router)   │                      │ (Renderer) │
                                 └──────────────┘                      └────────────┘
```

## Phase 1: Event System Foundation (Week 1)

### 1.1 Create Event Infrastructure

**File**: `src/repl/events.rs`
```rust
// Core event types
#[derive(Debug, Clone)]
pub enum ModelEvent {
    // Cursor events
    CursorMoved { 
        pane: Pane, 
        old_pos: LogicalPosition, 
        new_pos: LogicalPosition 
    },
    
    // Content events
    TextInserted { 
        pane: Pane, 
        position: LogicalPosition, 
        text: String 
    },
    TextDeleted { 
        pane: Pane, 
        range: LogicalRange 
    },
    LineInserted { 
        pane: Pane, 
        line: usize 
    },
    LineDeleted { 
        pane: Pane, 
        line: usize 
    },
    
    // Mode events
    ModeChanged { 
        from: EditorMode, 
        to: EditorMode 
    },
    
    // Pane events
    PaneSwitched { 
        from: Pane, 
        to: Pane 
    },
    
    // Request/Response events
    RequestExecuted,
    ResponseReceived { 
        status: StatusCode, 
        body: String 
    },
}

#[derive(Debug, Clone)]
pub enum ViewModelEvent {
    // Display updates
    DisplayCacheUpdated { pane: Pane },
    ScrollPositionChanged { 
        pane: Pane, 
        old_offset: usize, 
        new_offset: usize 
    },
    
    // Render hints
    FullRedrawRequired,
    PaneRedrawRequired { pane: Pane },
    StatusBarUpdateRequired,
    CursorRepositionRequired,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress(KeyEvent),
    Resize { width: u16, height: u16 },
}

// Event bus trait
pub trait EventBus {
    fn publish(&mut self, event: ModelEvent);
    fn subscribe_model(&mut self, handler: Box<dyn Fn(&ModelEvent)>);
    fn subscribe_view(&mut self, handler: Box<dyn Fn(&ViewModelEvent)>);
}
```

### 1.2 Implement Simple Event Bus

**File**: `src/repl/event_bus.rs`
```rust
pub struct SimpleEventBus {
    model_handlers: Vec<Box<dyn Fn(&ModelEvent)>>,
    view_handlers: Vec<Box<dyn Fn(&ViewModelEvent)>>,
}

impl SimpleEventBus {
    pub fn new() -> Self {
        Self {
            model_handlers: Vec::new(),
            view_handlers: Vec::new(),
        }
    }
}

impl EventBus for SimpleEventBus {
    fn publish(&mut self, event: ModelEvent) {
        for handler in &self.model_handlers {
            handler(&event);
        }
    }
    
    // ... other methods
}
```

## Phase 2: Pure Model Layer (Week 2)

### 2.1 Extract Pure Model

**File**: `src/repl/model/pure_model.rs`
```rust
// Pure model with only logical state
#[derive(Debug, Clone)]
pub struct PureModel {
    pub request_content: BufferContent,
    pub response_content: BufferContent,
    pub request_cursor: LogicalPosition,
    pub response_cursor: LogicalPosition,
    pub current_pane: Pane,
    pub mode: EditorMode,
    pub command_buffer: String,
    pub undo_stack: UndoStack,
}

#[derive(Debug, Clone)]
pub struct BufferContent {
    lines: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct LogicalPosition {
    pub line: usize,
    pub column: usize,
}

impl PureModel {
    // Only logical operations
    pub fn move_cursor(&mut self, pane: Pane, new_pos: LogicalPosition) -> ModelEvent {
        let old_pos = match pane {
            Pane::Request => self.request_cursor,
            Pane::Response => self.response_cursor,
        };
        
        match pane {
            Pane::Request => self.request_cursor = new_pos,
            Pane::Response => self.response_cursor = new_pos,
        }
        
        ModelEvent::CursorMoved { pane, old_pos, new_pos }
    }
    
    pub fn insert_text(&mut self, pane: Pane, text: &str) -> Vec<ModelEvent> {
        // Pure text insertion logic
        let mut events = vec![];
        
        // ... insertion logic ...
        
        events.push(ModelEvent::TextInserted {
            pane,
            position: self.get_cursor(pane),
            text: text.to_string(),
        });
        
        events
    }
}
```

### 2.2 Refactor Existing Model

Create adapter to maintain backward compatibility while transitioning:

**File**: `src/repl/model/adapter.rs`
```rust
// Temporary adapter during transition
pub struct ModelAdapter {
    pure_model: PureModel,
    // Legacy fields for compatibility
    pub request_buffer: RequestBuffer,
    pub response_buffer: ResponseBuffer,
}

impl ModelAdapter {
    pub fn sync_from_legacy(&mut self) {
        // Sync pure model from legacy buffers
    }
    
    pub fn sync_to_legacy(&mut self) {
        // Sync legacy buffers from pure model
    }
}
```

## Phase 3: ViewModel Implementation (Week 3)

### 3.1 Create ViewModel

**File**: `src/repl/view_model.rs`
```rust
pub struct ViewModel {
    // Display state
    request_display: DisplayState,
    response_display: DisplayState,
    
    // Display cache (moved from Model)
    display_cache: DisplayCache,
    
    // Terminal dimensions
    terminal_size: (u16, u16),
    pane_heights: PaneHeights,
    
    // Event bus reference
    event_bus: Arc<Mutex<dyn EventBus>>,
}

#[derive(Debug, Clone)]
pub struct DisplayState {
    // Display positions
    display_cursor: DisplayPosition,
    scroll_offset: usize,
    
    // Wrapped lines info
    wrapped_lines: Vec<WrappedLine>,
    
    // Selection (future feature)
    selection: Option<DisplayRange>,
}

impl ViewModel {
    pub fn new(event_bus: Arc<Mutex<dyn EventBus>>) -> Self {
        let mut vm = Self {
            // ... initialization
            event_bus,
        };
        
        // Subscribe to model events
        vm.subscribe_to_model_events();
        vm
    }
    
    fn subscribe_to_model_events(&mut self) {
        let weak_self = Arc::downgrade(&Arc::new(Mutex::new(self.clone())));
        
        self.event_bus.lock().unwrap().subscribe_model(Box::new(move |event| {
            if let Some(vm) = weak_self.upgrade() {
                vm.lock().unwrap().handle_model_event(event);
            }
        }));
    }
    
    fn handle_model_event(&mut self, event: &ModelEvent) {
        match event {
            ModelEvent::CursorMoved { pane, old_pos, new_pos } => {
                self.update_display_cursor(*pane, *new_pos);
                self.check_auto_scroll(*pane);
            }
            
            ModelEvent::TextInserted { pane, position, text } => {
                self.update_display_cache(*pane);
                self.recalculate_wrapping(*pane);
            }
            
            // ... other events
        }
    }
    
    fn update_display_cursor(&mut self, pane: Pane, logical_pos: LogicalPosition) {
        let display_state = match pane {
            Pane::Request => &mut self.request_display,
            Pane::Response => &mut self.response_display,
        };
        
        // Convert logical to display position
        if let Some(display_pos) = self.display_cache.logical_to_display(logical_pos) {
            display_state.display_cursor = display_pos;
            
            // Emit view event
            self.emit_view_event(ViewModelEvent::CursorRepositionRequired);
        }
    }
    
    fn check_auto_scroll(&mut self, pane: Pane) {
        let display_state = match pane {
            Pane::Request => &mut self.request_display,
            Pane::Response => &mut self.response_display,
        };
        
        let visible_height = self.get_visible_height(pane);
        let cursor_line = display_state.display_cursor.line;
        
        // Auto-scroll logic (moved from Commands)
        if cursor_line < display_state.scroll_offset {
            display_state.scroll_offset = cursor_line;
            self.emit_view_event(ViewModelEvent::ScrollPositionChanged {
                pane,
                old_offset: display_state.scroll_offset,
                new_offset: cursor_line,
            });
        } else if cursor_line >= display_state.scroll_offset + visible_height {
            let new_offset = cursor_line - visible_height + 1;
            let old_offset = display_state.scroll_offset;
            display_state.scroll_offset = new_offset;
            self.emit_view_event(ViewModelEvent::ScrollPositionChanged {
                pane,
                old_offset,
                new_offset,
            });
        }
    }
}
```

## Phase 4: Simplify Commands (Week 4)

### 4.1 Refactor Movement Commands

**Before** (current implementation):
```rust
impl Command for MoveCursorUpCommand {
    fn process(&self, state: &mut AppState) -> Result<()> {
        // Complex logic with display cache interaction
        let cache = match state.current_pane {
            Pane::Request => state.cache_manager.get_request_cache(),
            Pane::Response => state.cache_manager.get_response_cache(),
        };
        
        // Convert positions, handle scrolling, etc.
        // ... 50+ lines of view logic ...
    }
}
```

**After** (pure command):
```rust
impl Command for MoveCursorUpCommand {
    fn process(&self, model: &mut PureModel, event_bus: &mut dyn EventBus) -> Result<()> {
        let pane = model.current_pane;
        let current_pos = model.get_cursor(pane);
        
        if current_pos.line > 0 {
            let new_pos = LogicalPosition {
                line: current_pos.line - 1,
                column: current_pos.column,
            };
            
            // Simple state change
            let event = model.move_cursor(pane, new_pos);
            event_bus.publish(event);
        }
        
        Ok(())
    }
}
```

### 4.2 Refactor Editing Commands

**Before**:
```rust
impl Command for InsertTextCommand {
    fn process(&self, state: &mut AppState) -> Result<()> {
        // Insert text
        // Update display cache
        // Handle auto-scrolling
        // ... complex view logic ...
    }
}
```

**After**:
```rust
impl Command for InsertTextCommand {
    fn process(&self, model: &mut PureModel, event_bus: &mut dyn EventBus) -> Result<()> {
        if model.mode != EditorMode::Insert {
            return Ok(());
        }
        
        let events = model.insert_text(model.current_pane, &self.text);
        
        for event in events {
            event_bus.publish(event);
        }
        
        Ok(())
    }
}
```

## Phase 5: Update Controller (Week 5)

### 5.1 Refactor ReplController

**File**: `src/repl/controller.rs`
```rust
pub struct ReplController {
    model: PureModel,
    view_model: ViewModel,
    view: ViewManager,
    event_bus: Arc<Mutex<SimpleEventBus>>,
    command_registry: CommandRegistry,
}

impl ReplController {
    pub async fn run(&mut self) -> Result<()> {
        // Initial render
        self.view_model.calculate_initial_state(&self.model);
        self.view.render(&self.view_model)?;
        
        loop {
            // Get input
            if let Some(input_event) = self.view.get_input_event()? {
                self.handle_input_event(input_event)?;
            }
            
            // Process any pending view events
            self.process_view_events()?;
        }
    }
    
    fn handle_input_event(&mut self, event: InputEvent) -> Result<()> {
        match event {
            InputEvent::KeyPress(key) => {
                if let Some(command) = self.command_registry.get_command(key, self.model.mode) {
                    // Execute command with pure model
                    command.process(&mut self.model, self.event_bus.lock().unwrap().as_mut())?;
                }
            }
            
            InputEvent::Resize { width, height } => {
                self.view_model.update_terminal_size(width, height);
                self.event_bus.lock().unwrap().publish_view(
                    ViewModelEvent::FullRedrawRequired
                );
            }
        }
        
        Ok(())
    }
}
```

## Phase 6: Integration and Testing (Week 6)

### 6.1 Update Integration Tests

Create test helpers that work with the new architecture:

**File**: `tests/helpers/test_event_bus.rs`
```rust
pub struct TestEventBus {
    pub model_events: Vec<ModelEvent>,
    pub view_events: Vec<ViewModelEvent>,
}

impl TestEventBus {
    pub fn assert_model_event(&self, expected: ModelEvent) {
        assert!(self.model_events.contains(&expected));
    }
}
```

### 6.2 Migration Checklist

1. **Commands to Migrate** (Priority Order):
   - [ ] Movement commands (h,j,k,l)
   - [ ] Word movement (w,b)
   - [ ] Line movement (0,$,gg,G)
   - [ ] Insert mode commands (i,a,o)
   - [ ] Deletion commands (x,dd)
   - [ ] Undo/Redo
   - [ ] Mode transitions
   - [ ] Command line (:q, :w)

2. **Tests to Update**:
   - [ ] Unit tests for pure model
   - [ ] Unit tests for view model
   - [ ] Integration tests with new event flow
   - [ ] Cucumber features (should mostly still pass)

3. **Cleanup Tasks**:
   - [ ] Remove display positions from Model
   - [ ] Remove scroll_offset from buffers
   - [ ] Move display_cache to ViewModel
   - [ ] Remove view dependencies from Commands
   - [ ] Delete legacy adapter code

## Benefits of This Architecture

1. **Clear Separation**: Commands only know about logical operations
2. **Testability**: Can test commands without any view logic
3. **Flexibility**: Easy to add new view modes or display strategies
4. **Performance**: Display calculations happen only when needed
5. **Maintainability**: Each component has a single responsibility

## Migration Strategy

### Week 1: Foundation
- Implement event system
- Create pure model structures
- Set up basic event bus

### Week 2: Model Layer
- Extract pure model
- Create model adapter
- Update model operations

### Week 3: ViewModel
- Implement ViewModel
- Move display cache
- Handle model events

### Week 4: Commands
- Refactor commands one by one
- Remove view logic
- Update command tests

### Week 5: Integration
- Update controller
- Fix integration tests
- Ensure feature parity

### Week 6: Cleanup
- Remove legacy code
- Update documentation
- Performance testing

## Example: Complete Flow for Cursor Movement

1. **User presses 'j'**
2. **View** captures input, sends `InputEvent::KeyPress('j')`
3. **Controller** looks up command for 'j' in normal mode
4. **MoveCursorDownCommand** executes:
   ```rust
   model.move_cursor_down() -> ModelEvent::CursorMoved
   ```
5. **EventBus** publishes `ModelEvent::CursorMoved`
6. **ViewModel** receives event:
   - Calculates new display position
   - Checks if scrolling needed
   - Emits `ViewModelEvent::CursorRepositionRequired`
7. **View** receives event and updates cursor position on screen

This flow ensures complete separation between logical operations and display concerns.