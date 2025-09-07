# BlueLine System Architecture

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Models Layer](#models-layer)
4. [ViewModels Layer](#viewmodels-layer)
5. [Views Layer](#views-layer)
6. [Commands System](#commands-system)
7. [Services Layer](#services-layer)
8. [Event System](#event-system)
9. [Data Flow](#data-flow)
10. [Key Design Principles](#key-design-principles)

## Architecture Overview

BlueLine follows a clean **Model-View-ViewModel (MVVM)** architecture pattern with clear separation of concerns and event-driven communication between components.

```text
┌─────────────┐     Events      ┌──────────────┐     Events      ┌──────────────┐
│   Models    │ ──────────────> │  ViewModels  │ ──────────────> │    Views     │
│ (Pure Data) │                 │(Orchestration)│                 │  (Rendering) │
└─────────────┘                 └──────────────┘                 └──────────────┘
       ↑                                │                                ↓
       │                                │                                │
       └────────── Commands ────────────┴──────── User Input ───────────┘
```

## Core Components

### Models (`src/repl/models/`)
**Pure data structures and business logic**
- `AppState`: Central application state containing all data
  - Merged from former ViewModel and PaneManager for simplified architecture
  - Contains request/response buffers, pane management, settings
- `PaneState`: Individual pane state (cursor, selection, mode)
- `BufferModel`: Text buffer with multi-byte character support
- `Settings`: Shared enums for settings and movement directions
- `Events`: Event types for inter-component communication

### ViewModels (`src/repl/view_models/`)
**Orchestration and coordination layer**
- `AppViewModel`: Main MVVM orchestrator
  - Manages the event loop
  - Coordinates between View and Model layers
  - Handles command execution via the unified command system
  - Integrates with services (HTTP, clipboard, etc.)
- `Commands`: Unified command system (moved from `unified_commands`)
  - Self-registering commands using inventory system
  - Event-driven command execution
  - Support for modal editing (Normal, Insert, Visual, Command modes)
- `PostCommandActions`: Events emitted after command execution

### Views (`src/repl/views/`)
**Rendering and display layer**
- `ViewRenderer`: Trait defining view rendering interface
- `TerminalRenderer`: Terminal-based implementation
  - Renders panes, status bar, and command line
  - Handles terminal-specific display logic
  - Efficient partial updates to minimize flickering

### Commands (`src/repl/view_models/commands/`)
**Unified command system for handling user input**
- **Dynamic Registry**: Auto-discovers and registers commands at compile time
- **Command Categories**:
  - `editing/`: Text manipulation (insert, delete, cut, paste)
  - `mode/`: Mode transitions (Normal, Insert, Visual, Command)
  - `navigation/`: Cursor and viewport movement
  - `system/`: Application control (quit, settings, HTTP requests)
  - `visual/`: Visual mode operations
  - `yank/`: Copy/paste operations
- **Key Features**:
  - Commands emit PostCommandActions rather than directly modifying state
  - Self-registration using inventory crate
  - Supports complex key sequences and modal editing

### Services (`src/repl/services/`)
**Business logic and external integrations**
- `HttpService`: HTTP request execution
- `YankService`: Clipboard operations (system and internal)
- `WordSegmenter`: Text boundary detection for navigation

### I/O Layer (`src/repl/io/`)
**Input/output abstractions**
- `EventStream`: Abstraction for input events
- `RenderStream`: Abstraction for rendering output
- Supports both terminal and mock implementations for testing

## Models Layer

The Models layer contains pure data structures representing application state:

### AppState (`src/repl/models/app_state/`)
Central application state with modular organization:
- **Core State Management**: Mode, panes, settings
- **Buffer Operations**: Text manipulation, visual selections
- **Cursor Management**: Movement and positioning
- **Display Management**: Line rendering, wrapping
- **Ex Command Manager**: Command mode operations
- **Pane Manager**: Multi-pane layout and focus

### Key Model Characteristics
1. **Pure Data**: No UI logic or rendering concerns
2. **Event Emission**: State changes generate events
3. **Multi-byte Support**: Full Unicode text handling
4. **Logical Coordinates**: Document-based positioning

## ViewModels Layer

The ViewModels layer orchestrates the application:

### AppViewModel
```rust
pub struct AppViewModel<ES: EventStream, RS: RenderStream> {
    app_state: AppState,
    view_renderer: TerminalRenderer<RS>,
    services: Services,
    unified_command_registry: DynamicCommandRegistry,
    event_bus: SimpleEventBus,
    event_stream: ES,
    // ... other fields
}
```

### Key Responsibilities
1. **Event Loop Management**: Process input, execute commands, render output
2. **Command Routing**: Match keys to commands based on current mode
3. **Service Integration**: Coordinate HTTP, clipboard, and other services
4. **State Coordination**: Synchronize model updates with view rendering

## Views Layer

The Views layer handles all rendering:

### TerminalRenderer
- **Double Buffering**: Compare screen states to minimize updates
- **Partial Rendering**: Update only changed regions
- **Cursor Management**: Hide during updates to prevent flickering
- **Status Bar**: Mode indicators, position, messages

## Commands System

The unified command system provides flexible, extensible input handling:

### Command Structure
```rust
pub trait Command: Send + Sync {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool;
    fn execute(&self, key_event: KeyEvent, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>>;
    fn name(&self) -> &'static str;
}
```

### Command Registration
Commands self-register using the inventory system:
```rust
register_command!(YankSelectionCommand, "YankSelectionCommand");
```

### Command Categories

#### Editing Commands
- Insert/delete characters and lines
- Cut/copy/paste operations
- Multi-cursor support for Visual Block mode

#### Navigation Commands
- Cursor movement (h, j, k, l)
- Word navigation (w, b, e)
- Page navigation (Ctrl+D, Ctrl+U)
- Line navigation (0, $, gg, G)

#### Mode Commands
- Mode transitions (i, a, v, Esc)
- Visual mode variants (V, Ctrl+V)
- Command mode (:)

#### System Commands
- Application control (:q, Ctrl+C)
- Settings management (:set wrap, :set number)
- HTTP request execution (Ctrl+Enter)

## Services Layer

Services encapsulate business logic and external integrations:

### HttpService
- Profile-based configuration
- Session header management
- Async request execution
- Response streaming

### YankService
- System clipboard integration (optional)
- Internal yank buffer
- Line-wise and character-wise yanking
- Visual mode support

## Event System

Events enable decoupled communication between components:

### PostCommandActions
```rust
pub enum PostCommandAction {
    FullRedrawRequired,
    CurrentAreaRedrawRequired,
    StatusBarUpdateRequired,
    ActiveCursorUpdateRequired,
    ModeChanged { new_mode: EditorMode },
    ExecuteHttpRequest,
    Quit,
    // ... more actions
}
```

### Event Flow
1. User input → Command execution
2. Command → PostCommandActions
3. AppViewModel processes actions
4. State updates → View rendering

## Data Flow

### Typical Command Execution

1. **Input Capture**: Terminal event stream provides KeyEvent
2. **Command Lookup**: Registry finds relevant command for key + mode
3. **Command Execution**: Command modifies AppState via ExecutionContext
4. **Action Emission**: Command returns PostCommandActions
5. **Action Processing**: AppViewModel handles actions
6. **Rendering**: View updates based on state changes

### Example: Text Insertion
```
User types 'a' in Insert mode
    ↓
InsertCharCommand.is_relevant() → true
    ↓
InsertCharCommand.execute()
    ↓
AppState.insert_char('a')
    ↓
Returns [CurrentAreaRedrawRequired, ActiveCursorUpdateRequired]
    ↓
AppViewModel processes actions
    ↓
TerminalRenderer updates display
```

## Key Design Principles

### 1. Separation of Concerns
- **Models**: Pure data and business logic
- **ViewModels**: Orchestration and coordination
- **Views**: Rendering and display
- **Commands**: Input handling
- **Services**: External integrations

### 2. Event-Driven Architecture
- Components communicate through events
- Loose coupling between layers
- Reactive UI updates

### 3. Testability
- Dependency injection (EventStream, RenderStream)
- Mock implementations for testing
- Pure functions where possible

### 4. Extensibility
- Self-registering command system
- Plugin-like architecture for new features
- Clear extension points

### 5. Performance
- Efficient partial rendering
- Display caching for wrapped text
- Minimal allocations in hot paths

### 6. Multi-byte Character Support
- Full Unicode support throughout
- Correct handling of wide characters
- Proper text boundary detection

## Recent Refactoring

### Command System Migration (Phase 4)
The architecture recently underwent major cleanup:
1. **Removed old command system** completely
2. **Unified commands** became the sole command system
3. **Moved to view_models**: Commands now at `src/repl/view_models/commands/`
4. **Simplified AppViewModel**: Removed ~250 lines of legacy code
5. **Cleaned up imports**: Consistent paths throughout codebase

### Architecture Improvements
- **Cleaner separation**: Commands clearly part of ViewModel layer
- **Reduced complexity**: Single command system instead of dual
- **Better organization**: Commands alongside their orchestrator
- **Improved maintainability**: Less code, clearer structure

## Benefits of This Architecture

1. **Maintainability**: Clear component boundaries and responsibilities
2. **Testability**: Mockable interfaces and dependency injection
3. **Performance**: Efficient rendering and minimal updates
4. **Extensibility**: Easy to add new commands and features
5. **Debuggability**: Event-driven flow provides clear audit trail
6. **International Support**: Full Unicode/multi-byte character handling
7. **Modal Editing**: Vim-style efficiency with clear mode separation

This MVVM architecture enables BlueLine to provide a responsive, efficient terminal-based HTTP client with vim-like editing capabilities while maintaining clean, testable, and maintainable code.