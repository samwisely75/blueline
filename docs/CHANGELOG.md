# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.1] - 2025-07-28

### Changed
- **Code Organization**: Renamed `movement.rs` to `navigation.rs` to align with Vim terminology
- **Module Structure**: Updated imports and declarations to reflect navigation command categorization

### Technical
- Maintains full backward compatibility
- All existing navigation commands (h/j/k/l) continue to work unchanged
- Improved code clarity through consistent terminology alignment

## [0.14.0] - 2025-07-28

### Added
- **MVVM Architecture**: Complete restructuring from MVC to Model-View-ViewModel pattern
- **Comprehensive View Model Layer**: Specialized managers for better separation of concerns
  - Core ViewModel with central coordination
  - Cursor Manager for position tracking and movement
  - Display Manager for rendering coordination
  - Pane Manager for layout management
  - HTTP Manager for request handling
  - Rendering Coordinator for optimized updates
  - Screen Buffer for double buffering support
- **Buffer Operations**: Text manipulation functionality with insert, delete operations
- **Ghost Cursor Prevention**: Throttled rendering and improved cursor visibility
- **Position Indicator Events**: Minimal status bar updates for better performance
- **Comprehensive Unit Tests**: Full test coverage for all view model components
- **Developer Workflow**: 14-step development process documentation

### Changed
- **Controller Updates**: Improved rendering with throttling and flickering reduction
- **Event System**: Enhanced view events with position indicator support
- **Display Coordination**: Better cursor synchronization between logical and display positions

### Preserved
- **Legacy Code**: Renamed `view_models.rs` to `view_models_old.rs` for reference

### Technical
- Maintains compatibility with existing crossterm-based terminal interface
- Improved modularity and testability through MVVM pattern
- Enhanced performance through selective rendering and double buffering

## [0.13.0] - Previous Release
- Horizontal scrolling implementation
- Flickering reduction improvements