# Comprehensive Refactoring Plan for Issue #178

## Executive Summary
Complete architectural refactoring to centralize control in ViewModel while maintaining clean separation of concerns through Command Pattern and event-driven rendering.

## Final Architecture Overview

```
Input Events → AppController → Commands → Model Events → Render Events → ViewRenderer
                     ↓             ↓           ↓             ↓
              CommandProcessor  Business   ViewModel    RenderLoop
                               Logic      (State Mgmt)  (Display)
```

## Core Principles

1. **Commands own business logic** - not moved to ViewModel
2. **ViewModel manages state only** - pure state management
3. **Model Events describe changes** - semantic, not visual
4. **Render Events specify display** - generated from Model Events
5. **No coupling between ViewRenderer and ViewModel**

## Phase 1: Create Command Infrastructure

### 1.1 Architectural Decisions (Updated)

**Selection as Encapsulated Object:**
- Selection is a proper object owned by PaneState
- Contains only positional data (start, end) - no mode or pane references
- Methods receive EditorMode as parameter for mode-specific behavior
- PaneState provides buffer-aware selection operations

**Command Pattern Foundation:**
```rust
// Command trait with business logic
trait Command {
    fn handle(&self, context: &mut CommandContext) -> Result<Vec<ModelEvent>>;
}

// Context provides access to ViewModel (no services layer initially)
struct CommandContext<'a> {
    view_model: &'a mut ViewModel,
}
```

### 1.2 Selection Object Design

```rust
// Pure positional selection data
struct Selection {
    start: LogicalPosition,
    end: LogicalPosition,
}

impl Selection {
    // Pure position operations - no mode knowledge
    pub fn normalize(&self) -> (LogicalPosition, LogicalPosition);
    pub fn contains(&self, pos: LogicalPosition) -> bool;
}

// PaneState owns selection and provides buffer-aware operations
impl PaneState {
    pub fn get_selected_text(&self) -> Option<String> {
        self.selection.as_ref().map(|sel| {
            match self.editor_mode {
                EditorMode::VisualLine => self.extract_line_selection(sel),
                EditorMode::VisualBlock => self.extract_block_selection(sel),
                _ => self.extract_char_selection(sel),
            }
        })
    }
}
```

### 1.3 Services Decision

**No Services Layer Initially:**
- Keep existing YankBuffer (already well-designed with polymorphism)
- Focus on proper object encapsulation rather than service abstraction
- Services can be added later when actual duplication emerges

### 1.3 Model Events (Semantic)

```rust
enum ModelEvent {
    // Text operations
    TextInserted { pane: Pane, position: Position, text: String },
    TextDeleted { pane: Pane, range: Range },
    
    // Cursor operations
    CursorMoved { pane: Pane, new_position: Position },
    
    // Selection operations
    SelectionChanged { pane: Pane, old_selection: Option<Selection>, new_selection: Option<Selection> },
    SelectionCleared { pane: Pane },
    
    // Mode changes
    ModeChanged { old_mode: EditorMode, new_mode: EditorMode },
    
    // Pane operations
    PaneSwitched { from: Pane, to: Pane },
    
    // Status updates
    StatusMessageSet { message: String },
    StatusMessageCleared,
    
    // HTTP operations
    HttpRequestStarted,
    HttpResponseReceived { status: u16, body: String },
    HttpRequestFailed { error: String },
}
```

## Phase 2: Migrate Business Logic to Commands

### 2.1 Example Commands

```rust
// Simple command example
struct YankSelectionCommand;
impl Command for YankSelectionCommand {
    fn handle(&self, ctx: &mut CommandContext) -> Result<Vec<ModelEvent>> {
        if let Some(text) = ctx.selection_service.get_selected_text(ctx.view_model) {
            let yank_type = ctx.selection_service.determine_yank_type(ctx.view_model.mode());
            
            // Store in yank service
            ctx.yank_service.yank(text.clone(), yank_type)?;
            
            // Clear selection and update mode
            ctx.selection_service.clear_selection(ctx.view_model)?;
            
            Ok(vec![
                ModelEvent::ModeChanged { 
                    old_mode: ctx.view_model.mode(), 
                    new_mode: EditorMode::Normal 
                },
                ModelEvent::SelectionCleared { pane: ctx.view_model.current_pane() },
                ModelEvent::StatusMessageSet { 
                    message: format!("{} characters yanked", text.len()) 
                },
            ])
        } else {
            Ok(vec![
                ModelEvent::StatusMessageSet { 
                    message: "No text selected".to_string() 
                }
            ])
        }
    }
}

// Complex command with async
struct ExecuteHttpRequestCommand {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}
impl Command for ExecuteHttpRequestCommand {
    fn handle(&self, ctx: &mut CommandContext) -> Result<Vec<ModelEvent>> {
        // Start execution
        let mut events = vec![ModelEvent::HttpRequestStarted];
        
        // Execute request
        match ctx.http_service.execute(&self.method, &self.url, &self.headers, &self.body).await {
            Ok(response) => {
                events.push(ModelEvent::HttpResponseReceived {
                    status: response.status,
                    body: response.body,
                });
            }
            Err(e) => {
                events.push(ModelEvent::HttpRequestFailed {
                    error: e.to_string(),
                });
            }
        }
        
        Ok(events)
    }
}
```

### 2.2 Migration Strategy

1. Start with simple commands: cursor movement, text insertion/deletion
2. Move to complex commands: yank/paste, visual operations, HTTP requests
3. Update `apply_command_event()` to create and execute Command objects
4. Delete old `handle_*` methods as they're migrated

### 2.3 Commands to Migrate from AppController

- `YankSelectionCommand` - from `handle_yank_selection()`
- `DeleteSelectionCommand` - from `handle_delete_selection()`
- `CutSelectionCommand` - from `handle_cut_selection()`
- `CutCharacterCommand` - from `handle_cut_character()`
- `CutToEndOfLineCommand` - from `handle_cut_to_end_of_line()`
- `CutCurrentLineCommand` - from `handle_cut_current_line()`
- `YankCurrentLineCommand` - from `handle_yank_current_line()`
- `ChangeSelectionCommand` - from `handle_change_selection()`
- `VisualBlockInsertCommand` - from `handle_visual_block_insert()`
- `VisualBlockAppendCommand` - from `handle_visual_block_append()`
- `ExitVisualBlockInsertCommand` - from `handle_exit_visual_block_insert()`
- `RepeatVisualSelectionCommand` - from `handle_repeat_visual_selection()`
- `PasteAfterCommand` - from `handle_paste_after()`
- `PasteAtCursorCommand` - from `handle_paste_at_cursor()`
- `ExecuteHttpRequestCommand` - from `handle_http_request()`
- `ChangeSettingCommand` - from `handle_setting_change()`
- `ShowProfileCommand` - from `handle_show_profile()`

## Phase 3: Merge PaneManager into ViewModel

### 3.1 Direct Pane Ownership

```rust
// ViewModel directly owns panes (no PaneManager layer)
struct ViewModel {
    panes: [PaneState; 2],  // Direct ownership
    current_pane: Pane,
    mode: EditorMode,
    status_line: StatusLine,
    
    // State management methods only
    pub fn current_pane(&self) -> &PaneState;
    pub fn current_pane_mut(&mut self) -> &mut PaneState;
    pub fn switch_pane(&mut self, pane: Pane);
    pub fn set_status_message(&mut self, msg: String);
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()>;
}
```

### 3.2 Migration Steps

1. Move `panes: [PaneState; 2]` directly into ViewModel
2. Move all PaneManager methods to ViewModel:
   - Pane switching logic
   - Terminal dimension calculations
   - Settings management (wrap, line numbers, tab width)
3. Update Commands to use new ViewModel structure
4. Delete PaneManager entirely

## Phase 4: Move Event Loop to ViewModel

### 4.1 ViewModel as Event Loop Owner

```rust
impl ViewModel {
    pub async fn run_event_loop(
        &mut self,
        event_stream: &mut impl EventStream,
        render_tx: Sender<RenderEvent>,
        command_processor: CommandProcessor,
        services: Services,
    ) -> Result<()> {
        while !self.should_quit {
            if let Some(event) = Self::poll_event(event_stream, Duration::from_millis(100))? {
                let model_events = self.process_event(event, command_processor, services).await?;
                
                // Translate Model Events to Render Events
                for model_event in model_events {
                    let render_events = self.translate_to_render_events(model_event);
                    for render_event in render_events {
                        render_tx.send(render_event).await?;
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn process_event(
        &mut self,
        event: Event,
        command_processor: CommandProcessor,
        mut services: Services,
    ) -> Result<Vec<ModelEvent>> {
        let commands = command_processor.process_event(event, self.mode);
        let mut all_events = Vec::new();
        
        for cmd in commands {
            let context = CommandContext {
                view_model: self,
                selection_service: &mut services.selection,
                yank_service: &mut services.yank,
                http_service: &mut services.http,
            };
            
            let events = cmd.handle(context)?;
            all_events.extend(events);
        }
        
        Ok(all_events)
    }
}
```

### 4.2 Minimized AppController

```rust
impl AppController {
    pub async fn run(&mut self) -> Result<()> {
        // Terminal setup
        self.view_renderer.initialize()?;
        
        // Create render channel
        let (render_tx, render_rx) = channel();
        
        // Run dual event loops
        tokio::select! {
            _ = self.view_model.run_event_loop(
                &mut self.event_stream, 
                render_tx,
                self.command_processor,
                self.services
            ) => {},
            _ = render_loop(render_rx, self.view_renderer) => {},
        }
        
        // Terminal cleanup
        self.view_renderer.cleanup()?;
        Ok(())
    }
}
```

## Phase 5: Decouple ViewRenderer

### 5.1 Render Events (Display-Specific)

```rust
enum RenderEvent {
    // Full pane render
    RenderPane {
        pane: Pane,
        display_lines: Vec<DisplayLine>,
        cursor_position: Position,
        selection: Option<Selection>,
        viewport: Viewport,
    },
    
    // Partial pane render - only specific lines
    RenderPanePartial {
        pane: Pane,
        start_line: usize,
        display_lines: Vec<DisplayLine>,
        cursor_position: Option<Position>,
        selection: Option<Selection>,
    },
    
    // Single line update (very efficient)
    RenderLine {
        pane: Pane,
        line_index: usize,
        display_line: DisplayLine,
        cursor_position: Option<Position>,
    },
    
    // Just move cursor (no content change)
    UpdateCursor {
        position: Position,
        style: CursorStyle,
    },
    
    // Just update selection highlighting
    UpdateSelection {
        pane: Pane,
        old_selection: Option<Selection>,
        new_selection: Option<Selection>,
    },
    
    RenderStatusBar {
        mode: String,
        message: String,
        position: String,
        profile: String,
    },
    
    RenderPositionIndicator { line: usize, column: usize },
}
```

### 5.2 Model Event Translation

```rust
struct RenderEventProducer;

impl RenderEventProducer {
    fn translate(&self, model_event: ModelEvent, view_model: &ViewModel) -> Vec<RenderEvent> {
        match model_event {
            ModelEvent::TextInserted { pane, position, text } => {
                if text.len() == 1 && !text.contains('\n') {
                    // Single character: render just one line
                    vec![RenderEvent::RenderLine {
                        pane,
                        line_index: position.line,
                        display_line: view_model.get_display_line(position.line),
                        cursor_position: Some(view_model.get_cursor_position()),
                    }]
                } else {
                    // Multi-line: partial render from insertion point
                    vec![RenderEvent::RenderPanePartial {
                        pane,
                        start_line: position.line,
                        display_lines: view_model.get_display_lines_from(position.line),
                        cursor_position: Some(view_model.get_cursor_position()),
                        selection: view_model.get_selection(),
                    }]
                }
            }
            ModelEvent::CursorMoved { new_position, .. } => {
                vec![RenderEvent::UpdateCursor {
                    position: new_position,
                    style: CursorStyle::from(view_model.get_mode()),
                }]
            }
            ModelEvent::SelectionChanged { pane, old_selection, new_selection } => {
                vec![RenderEvent::UpdateSelection {
                    pane,
                    old_selection,
                    new_selection,
                }]
            }
            ModelEvent::StatusMessageSet { message } => {
                vec![RenderEvent::RenderStatusBar {
                    mode: format!("{:?}", view_model.get_mode()),
                    message,
                    position: format!("{}:{}", view_model.get_cursor().line, view_model.get_cursor().column),
                    profile: view_model.get_profile_name().to_string(),
                }]
            }
            // ... other translations
        }
    }
}
```

### 5.3 ViewRenderer Independence

```rust
// ViewRenderer only knows about RenderEvents - NO ViewModel dependency
struct ViewRenderer {
    render_stream: RenderStream,
    screen_state: ScreenState,  // Tracks what's on screen
}

impl ViewRenderer {
    // No ViewModel reference - pure event processing
    pub async fn render(&mut self, event: RenderEvent) -> Result<()> {
        match event {
            RenderEvent::RenderLine { pane, line_index, display_line, cursor_position } => {
                self.render_line_at(pane, line_index, &display_line)?;
                if let Some(pos) = cursor_position {
                    self.position_cursor(pos)?;
                }
            }
            // ... handle other events
        }
    }
}
```

## Phase 6: Dual Event Loops with Ghost Cursor Fix

### 6.1 Concurrent Event Processing

```rust
async fn main() {
    let (render_tx, render_rx) = channel();
    
    tokio::select! {
        _ = input_and_command_loop(event_stream, view_model, render_tx) => {},
        _ = render_loop(render_rx, view_renderer) => {},
    }
}
```

### 6.2 Render Transactions (Atomic Updates)

```rust
// Render Transaction for atomic updates (solves ghost cursor)
struct RenderTransaction {
    hide_cursor: bool,
    operations: Vec<RenderOperation>,
    show_cursor_at: Option<Position>,
    flush: bool,
}

enum RenderOperation {
    WriteAt { position: Position, content: String },
    ClearLine { line: usize },
    ClearRegion { start: Position, end: Position },
    SetColors { fg: Color, bg: Color },
}

impl ViewRenderer {
    async fn render(&mut self, transaction: RenderTransaction) -> Result<()> {
        // CRITICAL: Hide cursor FIRST to prevent ghosts
        if transaction.hide_cursor {
            self.hide_cursor()?;
        }
        
        // Perform all operations without flushing
        for op in transaction.operations {
            self.apply_operation(op)?;  // Writes to buffer only
        }
        
        // Position cursor AFTER all content updates
        if let Some(pos) = transaction.show_cursor_at {
            self.move_cursor_to(pos)?;
            self.show_cursor()?;
        }
        
        // Single flush at the end - atomic update to screen
        if transaction.flush {
            self.render_stream.flush()?;
        }
        
        Ok(())
    }
}
```

### 6.3 Render Loop with Batching and Double Buffering

```rust
async fn render_loop(mut render_rx: Receiver<RenderEvent>, mut renderer: ViewRenderer) {
    const BATCH_TIMEOUT: Duration = Duration::from_micros(500);
    
    loop {
        let mut builder = RenderTransactionBuilder::new();
        let batch_start = Instant::now();
        
        // Collect events for up to 500μs to prevent flickering
        loop {
            match render_rx.try_recv() {
                Ok(event) => {
                    builder.add_event(event);
                    
                    // Keep collecting unless timeout
                    if batch_start.elapsed() >= BATCH_TIMEOUT {
                        break;
                    }
                }
                Err(_) => {
                    if builder.has_events() {
                        break;  // Render what we have
                    } else {
                        // Wait for next event
                        if let Some(event) = render_rx.recv().await {
                            builder.add_event(event);
                        }
                    }
                }
            }
        }
        
        // Build and execute transaction atomically
        let transaction = builder.build();
        renderer.render(transaction).await?;
    }
}

struct RenderTransactionBuilder {
    current_transaction: Option<RenderTransaction>,
}

impl RenderTransactionBuilder {
    fn add_event(&mut self, event: RenderEvent) {
        match event {
            RenderEvent::RenderPanePartial { .. } => {
                // Always hide cursor for content updates
                self.ensure_transaction().hide_cursor = true;
                self.add_content_operations(event);
            }
            
            RenderEvent::UpdateCursor { position, .. } => {
                // Set final cursor position
                self.ensure_transaction().show_cursor_at = Some(position);
            }
        }
    }
    
    fn build(mut self) -> RenderTransaction {
        let mut trans = self.current_transaction.unwrap_or_default();
        trans.flush = true;  // Always flush completed transaction
        trans
    }
}
```

### 6.4 Double Buffering

```rust
struct ViewRenderer {
    // Two buffers for smooth updates
    front_buffer: ScreenBuffer,
    back_buffer: ScreenBuffer,
    render_stream: RenderStream,
}

impl ViewRenderer {
    async fn render(&mut self, transaction: RenderTransaction) -> Result<()> {
        // 1. Hide cursor to prevent ghosts
        if transaction.hide_cursor {
            self.render_stream.execute(cursor::Hide)?;
        }
        
        // 2. Apply all changes to back buffer
        for op in transaction.operations {
            self.back_buffer.apply(op);
        }
        
        // 3. Diff and send only changes to terminal
        let changes = self.front_buffer.diff(&self.back_buffer);
        for change in changes {
            self.render_stream.queue(change)?;
        }
        
        // 4. Update cursor position
        if let Some(pos) = transaction.show_cursor_at {
            self.render_stream.queue(cursor::MoveTo(pos.col, pos.row))?;
            self.render_stream.queue(cursor::Show)?;
        }
        
        // 5. Single atomic flush to screen
        self.render_stream.flush()?;
        
        // 6. Swap buffers for next frame
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        
        Ok(())
    }
}
```

## Key Benefits

### Architecture
1. **Clean Separation**: Each component has single responsibility
2. **No Coupling**: ViewRenderer independent of ViewModel
3. **Testable**: Each layer independently testable
4. **Extensible**: Easy to add new commands or renderers
5. **Future-Ready**: AppController can be eliminated entirely

### Performance
1. **No Ghost Cursors**: Atomic render transactions
2. **No Flickering**: Double buffering and smart diffing
3. **Efficient Updates**: Only changed content is rendered
4. **Batched Rendering**: Multiple events combined into single update

### Maintainability
1. **Command Pattern**: Each operation is self-contained
2. **Event-Driven**: Clear data flow through events
3. **Service Layer**: Reusable business logic
4. **Minimal Controllers**: Thin coordination layers

## Implementation Order

### Week 1: Foundation (Phase 1-2)
- **Days 1-2**: Command infrastructure and services
- **Days 3-4**: Migrate simple commands (cursor, text operations)
- **Days 4-5**: Migrate complex commands (yank, paste, visual, HTTP)

### Week 2: Core Refactoring (Phase 3-4)
- **Day 1**: Merge PaneManager into ViewModel
- **Day 2**: Move event loop to ViewModel
- **Day 3**: Testing and bug fixes

### Week 3: Rendering (Phase 5-6)
- **Days 1-2**: Create RenderEvent system and decouple ViewRenderer
- **Days 2-3**: Implement dual event loops
- **Days 4-5**: Add render transactions and double buffering

Total estimated effort: **15 working days** (3 weeks)

## Success Metrics

### Code Quality
- AppController reduced from **1500+ to ~100 lines**
- Commands are self-contained units (**50-100 lines each**)
- ViewModel is pure state management (**~400 lines**)
- Zero coupling between ViewRenderer and ViewModel

### Performance
- **Zero ghost cursors** in all scenarios
- **No flickering** during rapid updates
- **Consistent 60fps** rendering performance
- **Sub-millisecond** input response time

### Architecture
- **Single Responsibility** principle followed by all components
- **Open/Closed** principle enables easy feature addition
- **Testability** - each layer can be unit tested independently
- **Maintainability** - clear data flow and minimal coupling

## Risk Mitigation

1. **Incremental Migration**: Each phase can be implemented and tested independently
2. **Backward Compatibility**: Keep existing tests passing during transition
3. **Feature Flags**: Can rollback individual phases if issues arise
4. **Comprehensive Testing**: Unit tests for each command and integration tests for event flow

This refactoring plan transforms the architecture into a clean, maintainable, and high-performance system while solving current issues like ghost cursors and providing a foundation for future enhancements.