# Architecture Overview

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Models Layer](#models-layer)
4. [ViewModel Layer](#viewmodel-layer)
5. [Event System](#event-system)
6. [Rendering System](#rendering-system)
7. [Data Flow](#data-flow)
8. [Key Patterns](#key-patterns)

## Architecture Overview

Blueline implements a **Model-View-ViewModel (MVVM)** architecture pattern enhanced with an **event-driven system** for reactive UI updates. This architecture provides clear separation of concerns between data management, display logic, and user interaction.

```text
┌─────────────┐     Commands     ┌──────────────┐     Events      ┌──────────────┐
│ Controller  │ ───────────────> │    Models    │ ──────────────> │  ViewModel   │
│  (Input)    │                  │ (Pure Data)  │                 │  (Display)   │
└─────────────┘                  └──────────────┘                 └──────────────┘
       ↑                                ↑                                │
       │                                │                                ↓
       │                                │         View Events     ┌──────────────┐
       └─────── User Input ─────────────┼────────────────────────│ Rendering    │
                                        │                        │   System     │
                                        │                        └──────────────┘
                                        │
                               ┌─────────────┐
                               │ Event Bus   │
                               │ (Pub/Sub)   │
                               └─────────────┘
```

### Design Principles

1. **Separation of Concerns**: Each layer has a distinct responsibility
2. **Event-Driven**: Components communicate through events, not direct coupling
3. **Reactive Updates**: UI automatically responds to model changes
4. **Testability**: Business logic is isolated from view concerns
5. **Performance**: Efficient partial updates minimize rendering overhead

## Core Components

### 1. Models Layer (`src/repl/models/`)

- **Pure data structures** containing application state
- **No view logic** or display calculations
- **Immutable operations** where possible
- **Domain-focused** business logic

### 2. ViewModel Layer (`src/repl/view_models/`)

- **Modular ViewModel architecture** split into focused responsibilities
- **PaneManager** for complete pane abstraction and semantic operations
- **Display state management** (scroll positions, cursor coordinates)
- **Coordinate transformations** (logical ↔ display positions)
- **Event handling** and propagation
- **Display cache management** for text wrapping and line mapping

### 3. Event System (`src/repl/events/`)

- **Model Events**: Data change notifications
- **View Events**: Rendering update requests
- **Input Events**: User interaction handling
- **Event Bus**: Central pub/sub coordinator

### 4. Rendering System (`src/repl/views/`)

- **Terminal output** using crossterm
- **Efficient partial updates** to minimize flickering
- **Cursor management** and styling
- **Status bar rendering**

## Models Layer

The Models layer contains pure data structures representing the application's business state.

### Core Models

#### BufferModel (`src/repl/models/buffer_model.rs`)

```rust
pub struct BufferModel {
    content: BufferContent,         // Text lines
    cursor: LogicalPosition,        // Cursor in logical coordinates
    pane: Pane,                    // Which pane this buffer belongs to
}
```

- Manages text content as logical lines
- Maintains cursor position in document coordinates
- Provides text manipulation operations (insert, delete)
- No awareness of display or wrapping

#### ResponseModel (`src/repl/models/response_model.rs`)

```rust
pub struct ResponseModel {
    status_code: Option<u16>,       // HTTP status code
    status_message: Option<String>, // HTTP status message
    body: String,                   // Response body content
    duration_ms: Option<u64>,       // Request duration
}
```

- Stores HTTP response data
- Tracks request execution timing
- Immutable once set

#### DisplayCache (`src/repl/models/display_cache.rs`)

```rust
pub struct DisplayCache {
    display_lines: Vec<DisplayLine>, // Wrapped/processed lines
    logical_mapping: Vec<LineMapping>, // Logical ↔ Display coordinate mapping
}
```

- Handles text wrapping and line breaking
- Provides coordinate conversion between logical and display positions
- Optimized for efficient lookups during rendering

### Key Model Characteristics

1. **Logical Coordinates**: Models work with document line/column positions
2. **Pure Functions**: Operations return events rather than mutating state
3. **Domain Focus**: Models understand business concepts, not UI concerns
4. **Event Emission**: State changes generate events for reactive updates

## ViewModel Layer

The ViewModel layer bridges the gap between pure model data and display requirements.

### Core ViewModel (`src/repl/view_models/core.rs`)

```rust
pub struct ViewModel {
    // Core state
    editor_mode: EditorMode,
    response: ResponseModel,
    
    // Pane management - complete abstraction through PaneManager
    pane_manager: PaneManager,
    
    // Status line model - encapsulates all status bar state
    status_line: StatusLine,
    
    // HTTP client and configuration
    http_client: Option<HttpClient>,
    http_session_headers: HashMap<String, String>,
    http_verbose: bool,
    
    // Event management
    event_bus: EventBusOption,
    pending_view_events: Vec<ViewEvent>,
    pending_model_events: Vec<ModelEvent>,
    
    // Double buffering for efficient rendering
    current_screen_buffer: ScreenBuffer,
    previous_screen_buffer: ScreenBuffer,
}
```

### PaneManager Architecture

The `PaneManager` encapsulates all pane-related state and operations, providing complete abstraction where external components never directly access pane-specific identifiers:

```rust
pub struct PaneManager {
    panes: [PaneState; 2],           // Private - no external access
    current_pane: Pane,
    wrap_enabled: bool,
    terminal_dimensions: (u16, u16),
    request_pane_height: u16,
}
```

#### PaneManager Responsibilities

1. **Complete Pane Abstraction**: External components use semantic operations like `switch_to_request_pane()` instead of pane indexing
2. **Pane Layout Management**: Calculates pane boundaries and heights based on terminal size
3. **Semantic Operations**: Provides domain-specific methods like `insert_char_in_request()`, `is_in_request_pane()`
4. **Display State Coordination**: Manages cursor positions, scroll offsets, and visual selection across panes
5. **Event Emission**: Returns semantic ViewEvents like `RequestContentChanged`, `FocusSwitched`

### PaneState Structure (Internal to PaneManager)

```rust
pub struct PaneState {
    buffer: BufferModel,                    // Text content and logical cursor
    display_cache: DisplayCache,            // Wrapped lines and coordinate mapping
    display_cursor: (usize, usize),         // Cursor in display coordinates
    scroll_offset: (usize, usize),          // Viewport scroll position
    visual_selection_start: Option<LogicalPosition>, // Visual mode selection
    visual_selection_end: Option<LogicalPosition>,
    pane_dimensions: (usize, usize),        // Pane width/height
}
```

### ViewModel Specialized Modules

The ViewModel is split into focused modules for better maintainability:

#### Core (`src/repl/view_models/core.rs`)

- Main ViewModel struct and basic initialization
- Terminal size management and screen buffer coordination
- Central coordinator that delegates to specialized managers

#### Mode Manager (`src/repl/view_models/mode_manager.rs`)

- Editor mode transitions (Normal, Insert, Visual, Command)
- Visual mode selection state management
- Mode-related event handling

#### Ex Command Manager (`src/repl/view_models/ex_command_manager.rs`)

- Ex command buffer operations (`:q`, `:set wrap`, etc.)
- Command parsing and execution
- Command mode state management

#### Buffer Operations (`src/repl/view_models/buffer_operations.rs`)

- Text insertion and deletion
- Character and line manipulation
- Display cache synchronization
- Event emission for content changes

#### Cursor Manager (`src/repl/view_models/cursor_manager.rs`)

- Cursor movement commands (up, down, left, right)
- Word navigation (w, b, e)
- Line navigation (0, $, gg, G)
- Auto-scrolling to keep cursor visible

#### Display Manager (`src/repl/view_models/display_manager.rs`)

- Display line rendering for terminal output
- Line number calculation and formatting
- Text wrapping and overflow handling
- Visual selection highlighting

#### PaneManager (`src/repl/view_models/pane_manager.rs`)

- **Complete pane abstraction** - external components never directly access pane arrays
- **Semantic operations** - `switch_to_request_pane()`, `insert_char_in_request()` instead of pane indexing
- **Pane layout management** - calculates boundaries, heights, and handles terminal resizing
- **Event abstraction** - emits semantic events like `RequestContentChanged`, `FocusSwitched`

#### Rendering Coordinator (`src/repl/view_models/rendering_coordinator.rs`)

- Screen buffer management for double buffering
- Differential rendering for performance
- View event emission and batching
- Render optimization strategies

### Key ViewModel Responsibilities

1. **Display State**: Manages cursor positions, scroll offsets, selection state
2. **Coordinate Translation**: Converts between logical and display coordinates
3. **Event Handling**: Responds to model events and emits view events
4. **Display Cache**: Maintains text wrapping and line mapping
5. **Auto-scrolling**: Ensures cursor remains visible during navigation
6. **Performance Optimization**: Uses double buffering and partial updates

## Event System

The event system enables reactive, decoupled communication between components.

### Event Types

#### Model Events (`src/repl/events/model_events.rs`)

```rust
pub enum ModelEvent {
    CursorMoved { pane: Pane, old_pos: LogicalPosition, new_pos: LogicalPosition },
    TextInserted { pane: Pane, position: LogicalPosition, text: String },
    TextDeleted { pane: Pane, range: LogicalRange },
    ModeChanged { old_mode: EditorMode, new_mode: EditorMode },
    PaneSwitched { old_pane: Pane, new_pane: Pane },
    RequestExecuted { method: String, url: String },
    ResponseReceived { status_code: u16, body: String },
}
```

#### View Events (`src/repl/events/view_events.rs`)

The ViewEvent system has been abstracted to eliminate pane parameter leakage:

```rust
pub enum ViewEvent {
    FullRedrawRequired,                                    // Complete screen refresh
    CurrentAreaRedrawRequired,                             // Redraw currently active pane
    SecondaryAreaRedrawRequired,                          // Redraw inactive pane
    ActiveCursorUpdateRequired,                           // Update cursor in active pane
    StatusBarUpdateRequired,                              // Status bar refresh
    PositionIndicatorUpdateRequired,                      // Position indicator only
    RequestContentChanged,                                // Request pane content changed
    ResponseContentChanged,                               // Response pane content changed
    FocusSwitched,                                        // Active pane changed
    CurrentAreaScrollChanged { old_offset: usize, new_offset: usize },
}
```

#### Input Events (`src/repl/events/view_events.rs`)

```rust
pub enum InputEvent {
    KeyPressed(KeyEvent),                     // User key input
    TerminalResized { width: u16, height: u16 }, // Terminal size change
}
```

### Event Bus (`src/repl/events/event_bus.rs`)

The EventBus provides a centralized pub/sub mechanism:

```rust
pub trait EventBus {
    fn emit_model_event(&mut self, event: ModelEvent);
    fn emit_view_event(&mut self, event: ViewEvent);
    fn subscribe_to_model_events(&mut self, handler: Box<dyn Fn(&ModelEvent)>);
    fn subscribe_to_view_events(&mut self, handler: Box<dyn Fn(&ViewEvent)>);
}
```

### Event Flow Patterns

1. **Command → Model Event**: Commands modify model state and emit events
2. **Model Event → ViewModel**: ViewModel reacts to model changes
3. **ViewModel → View Event**: ViewModel emits rendering instructions
4. **View Event → Rendering**: Terminal renderer updates display

## Rendering System

The rendering system efficiently updates the terminal display based on view events.

### Terminal Renderer (`src/repl/views/terminal_renderer.rs`)

```rust
pub struct TerminalRenderer {
    stdout: io::Stdout,
    terminal_size: (u16, u16),
}

pub trait ViewRenderer {
    fn render_full(&mut self, view_model: &ViewModel) -> Result<()>;
    fn render_pane(&mut self, view_model: &ViewModel, pane: Pane) -> Result<()>;
    fn render_pane_partial(&mut self, view_model: &ViewModel, pane: Pane, start_line: usize) -> Result<()>;
    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()>;
    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()>;
}
```

### Rendering Efficiency

1. **Partial Updates**: Only redraw changed areas
2. **Double Buffering**: Compare screen states to minimize updates
3. **Cursor Management**: Hide cursor during updates to prevent flickering
4. **Event-Driven**: React only to necessary changes

### Visual Features

1. **Line Numbers**: Dynamic width calculation and vim-style formatting
2. **Syntax Highlighting**: Prepared for HTTP syntax highlighting
3. **Visual Selection**: Text selection highlighting in visual mode
4. **Status Bar**: Mode indicators, HTTP status, cursor position
5. **Cursor Styles**: Different shapes for different modes (block, bar, underline)

## Data Flow

### Typical Command Execution Flow

1. **User Input**: Key press captured by controller
2. **Command Lookup**: Controller maps key to command based on current mode
3. **Command Execution**: Command modifies model state
4. **Model Event**: Command emits model event describing the change
5. **ViewModel Reaction**: ViewModel receives model event via event bus
6. **Display Update**: ViewModel calculates display changes and emits view events
7. **Rendering**: Terminal renderer processes view events and updates display

### Example: Cursor Movement Flow

```rust
// 1. User presses 'j' key
InputEvent::KeyPressed(KeyEvent { code: KeyCode::Char('j'), .. })

// 2. Controller maps to MoveCursorDownCommand
controller.handle_key_event('j') -> MoveCursorDownCommand

// 3. Command moves cursor in model
command.execute() -> model.move_cursor_down()

// 4. Model emits event
ModelEvent::CursorMoved { 
    pane: Pane::Request, 
    old_pos: LogicalPosition(5, 10), 
    new_pos: LogicalPosition(6, 10) 
}

// 5. ViewModel handles event
view_model.handle_cursor_moved() -> {
    // Convert to display coordinates
    // Check if scrolling needed
    // Update display cursor position
}

// 6. ViewModel emits view events
ViewEvent::CursorUpdateRequired { pane: Pane::Request }
ViewEvent::PositionIndicatorUpdateRequired  // Update status bar

// 7. Renderer updates display
terminal_renderer.render_cursor()
terminal_renderer.render_position_indicator()
```

## Key Patterns

### 1. Event-Driven Updates

- All component communication happens through events
- Enables loose coupling and reactive behavior
- Facilitates testing by capturing event streams

### 2. Coordinate System Separation

- **Logical Coordinates**: Document line/column positions (models)
- **Display Coordinates**: Terminal row/column positions (view model)
- **Viewport Coordinates**: Visible area relative positions (rendering)

### 3. Complete Pane Abstraction

- `PaneManager` provides complete encapsulation of pane-related state
- External components use semantic operations: `switch_to_request_pane()`, `is_in_request_pane()`
- ViewEvents use semantic naming: `RequestContentChanged`, `FocusSwitched` instead of pane parameters
- Eliminates pane leakage throughout the architecture

### 4. Partial Update Optimization

- Multiple view event granularities for efficiency
- `FullRedrawRequired` → `CurrentAreaRedrawRequired` → `ActiveCursorUpdateRequired` → `PositionIndicatorUpdateRequired`
- Semantic events reduce coupling and improve performance

### 5. Display Cache Management

- Pre-computed text wrapping and line mapping
- Efficient coordinate conversions during navigation
- Background updates for large content handling

### 6. Double Buffering

- `ScreenBuffer` abstraction for comparing render states
- Only update changed terminal regions
- Reduces flickering and improves perceived performance

## Benefits of This Architecture

1. **Testability**: Business logic separated from UI concerns
2. **Maintainability**: Modular ViewModel with clear component boundaries and responsibilities
3. **Encapsulation**: Complete pane abstraction eliminates component coupling
4. **Performance**: Efficient partial updates and semantic event system
5. **Flexibility**: Easy to add new features without affecting existing code
6. **Debuggability**: Event streams provide clear audit trail of state changes
7. **Extensibility**: Plugin-like architecture for adding new commands and features
8. **Domain-Driven Design**: Operations match domain concepts (Request vs Response panes)

This modular MVVM architecture with complete pane abstraction enables Blueline to provide a responsive, efficient terminal-based HTTP client with vim-like editing capabilities while maintaining clean, testable, and maintainable code.

