# BlueLine Architecture

## Overview

BlueLine follows a clean MVVM (Model-View-ViewModel) architecture pattern with clear separation of concerns.

## Core Components

### Models (`src/repl/models/`)
- **AppState** (`app_state/`): Central application state containing all data and business logic
  - Merged from former ViewModel and PaneManager
  - Contains request/response buffers, pane management, settings, and event handling
- **PaneState** (`pane_state/`): Individual pane state (cursor, selection, mode)
- **BufferModel** (`buffer/`): Text buffer implementation with multi-byte character support
- **Events** (`events.rs`): Event types for inter-component communication

### ViewModels (`src/repl/view_models/`)
- **AppViewModel** (`app_view_model.rs`): Main orchestrator of the MVVM pattern
  - Manages the event loop
  - Coordinates between View and Model layers
  - Handles command execution and event processing
  - Integrates with services (HTTP, clipboard, etc.)

### Views (`src/repl/views/`)
- **ViewRenderer** (`view_renderer.rs`): Trait defining view rendering interface
- **TerminalRenderer** (`terminal_renderer.rs`): Terminal-based implementation
  - Renders panes, status bar, and command line
  - Handles terminal-specific display logic

### Commands (`src/repl/commands/` and `src/repl/unified_commands/`)
- Two command systems working in parallel:
  - Legacy command system (being phased out)
  - Unified command system (new, more flexible)
- Commands emit events rather than directly modifying state
- Support for modal editing (Normal, Insert, Visual, Command modes)

### Services (`src/repl/services/`)
- **HttpService**: HTTP request execution
- **YankService**: Clipboard operations
- **DCutService**: Advanced cut operations

### I/O (`src/repl/io/`)
- **EventStream**: Abstraction for input events
- **RenderStream**: Abstraction for rendering output
- Supports both terminal and test implementations

## Data Flow

1. **Input**: User input → EventStream → AppViewModel
2. **Command Processing**: AppViewModel → Command Registry → Command execution
3. **State Updates**: Commands emit events → AppViewModel processes events → Updates AppState
4. **Rendering**: AppState changes → ViewRenderer reads AppState → Terminal output

## Key Design Principles

1. **Separation of Concerns**: Each layer has clear responsibilities
2. **Event-Driven**: Components communicate through events
3. **Testability**: Abstractions allow easy mocking and testing
4. **Multi-byte Support**: Full support for international characters
5. **Modal Editing**: Vim-style modal interface

## Recent Refactoring (Phases 1-7)

The architecture underwent major refactoring to achieve proper MVVM separation:

1. **Phase 1-2**: Prepared ViewRenderer to use AppState directly
2. **Phase 3-4**: Merged ViewModel and PaneManager into AppState
3. **Phase 5-6**: Moved AppController to ViewModels as AppViewModel
4. **Phase 7**: Cleanup and documentation

The result is a cleaner architecture where:
- AppState contains all application state and business logic
- AppViewModel orchestrates the MVVM pattern
- ViewRenderer only depends on AppState (not ViewModel)
- No controllers directory - everything properly organized in MVVM layers