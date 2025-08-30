# Blueline Refactoring Plan - MVVM Architecture

## Executive Summary

This document outlines the refactoring plan to transform Blueline into a proper MVVM (Model-View-ViewModel) architecture. The key insight driving this refactoring is that our current naming was incorrect:

- What we called "ViewModel" is actually the **Model** (pure state)
- What we called "AppController" is actually doing **ViewModel** work
- The architecture needs reorganization to properly separate concerns

## Architectural Vision

### Current (Incorrect) Architecture
```
View (Renderer) ← AppController (1500+ lines) ← ViewModel (actually just Model)
                         ↓
                    Services (stateful)
```

### Target MVVM Architecture
```
View (ViewRenderer) ← AppViewModel (consolidated) ← AppState (pure Model)
                            ↓
                       Services (stateful)
                            ↓
                       Commands (event-based)
```

## Key Architectural Decisions

1. **Rename for Clarity**: ViewModel → AppState, AppController → AppViewModel
2. **Event-Based Commands**: Revert from 3G (direct execution) back to 1G (event emission)
3. **State Consolidation**: Merge scattered state from AppController into AppViewModel
4. **Services Layer**: Extract stateful business logic into dedicated services
5. **ViewRenderer Enhancement**: Move screen buffers and rendering logic to View layer

## Refactoring Phases

### Phase 1: Reorganize Models
**Goal**: Clean up model organization for better maintainability

- Move models into logical subdirectories (commands/, services/, events/, etc.)
- Establish clear module boundaries
- No functional changes, pure reorganization

### Phase 2: Revert to Event-Based Commands
**Goal**: Restore proper MVVM pattern with event-driven architecture

- Revert Command trait from `execute()` back to `handle()` returning events
- Commands emit semantic events describing WHAT happened
- AppViewModel interprets events and updates state accordingly
- Maintains separation between business logic and state management

### Phase 3: Create Missing Services
**Goal**: Extract stateful business logic from AppController

- Create HttpService for HTTP request handling
- Create VisualBlockService for visual block operations
- Move stateful logic out of AppController into services
- Services handle complex operations, emit results via events

### Phase 4: Enhance ViewRenderer
**Goal**: Move rendering concerns to proper View layer

- Transfer screen buffers from AppController to ViewRenderer
- Move cursor management to ViewRenderer
- Extract all rendering logic from AppController
- ViewRenderer becomes sole owner of display state

### Phase 5: The Great Consolidation
**Goal**: Create proper AppViewModel by merging AppController and ViewModel

- Rename ViewModel → AppState (pure model)
- Rename AppController → AppViewModel
- Move all scattered state from old AppController into AppViewModel
- AppViewModel becomes the single source of truth for application state
- AppViewModel coordinates between AppState, Services, and Commands

### Phase 6: Modularize AppViewModel
**Goal**: Break down monolithic AppViewModel for maintainability

- Split AppViewModel into partial implementations by concern:
  - `app_view_model_navigation.rs`: Cursor and movement logic
  - `app_view_model_editing.rs`: Text manipulation logic
  - `app_view_model_visual.rs`: Visual mode logic
  - `app_view_model_commands.rs`: Command processing
  - `app_view_model_http.rs`: HTTP request coordination
- Each module handles specific aspect of ViewModel responsibilities
- Main AppViewModel file ties modules together

### Phase 7: Clean Up
**Goal**: Remove obsolete code and finalize architecture

- Delete old CommandRegistry
- Remove redundant event types
- Clean up unused imports and dead code
- Update all tests to use new architecture
- Document final architecture

## Migration Strategy

### Incremental Approach
Each phase should be completed in a separate PR to maintain stability:

1. Start with non-breaking changes (Phase 1: reorganization)
2. Gradually introduce new components alongside old ones
3. Switch over once new components are stable
4. Remove old components only after verification

### Testing Strategy
- Maintain existing integration tests throughout
- Add unit tests for new Services and Commands
- Verify no regression in functionality at each phase
- Manual testing of key workflows after each phase

## Success Criteria

The refactoring will be considered successful when:

1. AppViewModel is under 500 lines (down from 1500+)
2. Clear separation between Model, ViewModel, and View
3. All business logic extracted to Services or Commands
4. Event-driven architecture properly implemented
5. No functional regressions
6. Improved testability and maintainability

## Rollback Plan

If issues arise during refactoring:

1. Each phase is in its own branch/PR
2. Can revert individual phases without affecting others
3. Old architecture remains functional until Phase 5
4. Comprehensive test suite ensures functionality preserved

## Timeline Estimate

- Phase 1: 1 day (mechanical reorganization)
- Phase 2: 2 days (command system reversion)
- Phase 3: 2 days (service creation)
- Phase 4: 2 days (ViewRenderer enhancement)
- Phase 5: 3 days (the great consolidation)
- Phase 6: 2 days (modularization)
- Phase 7: 1 day (cleanup)

Total: ~2 weeks of focused development

## Notes

This plan represents a fundamental shift in understanding of the architecture. The key insight is that what we thought was MVVM was actually mislabeled, leading to confusion about where responsibilities should lie. By correcting the naming and structure, we achieve:

- Better separation of concerns
- Reduced coupling between components
- Improved testability
- Clearer mental model for future development

The event-based command system (1G) is actually the correct approach for MVVM, not the direct execution (3G) we initially pursued. Commands should describe intentions, not perform actions directly.