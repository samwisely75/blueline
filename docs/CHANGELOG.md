# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.34.0] - 2025-01-12

### Added

- **Config File Loading**: Load settings from `~/.blueline/config` file at startup (issue #19)
  - Config file uses ex command format (e.g., `set wrap on`, `set number on`)
  - Supports comments (lines starting with #) and empty lines
  - Environment variable `BLUELINE_CONFIG_PATH` can override default location
  - Commands are applied automatically at startup before UI is shown
  - Config errors are logged but don't crash the application

### Changed

- **Configuration Architecture**: Refactored to use unified `AppConfig` pattern
  - Consolidated all configuration sources into single `AppConfig` struct
  - Removed verbose flag from command line arguments (preparing for ex command implementation)
  - Replaced `dirs` crate with `shellexpand` for better path expansion
  - Cleaner separation of concerns for configuration management

## [0.33.0] - 2025-01-12

### Changed

- **Ex Command Architecture**: Introduced Command Pattern to ex commands (issue #64)
  - Created `ExCommand` trait and `ExCommandRegistry` for unified command handling
  - Separated command execution logic from StatusLine for better separation of concerns
  - Ex commands now use the same event-driven architecture as normal commands
  - Command buffer is properly cleared after execution with return to previous mode
  - Improved extensibility - new ex commands can be added by implementing the trait

## [0.32.1] - 2025-01-12

### Changed

- **Wrap Command Syntax**: Improved wrap command syntax to use `:set wrap on/off` (issue #17)
  - **Breaking Change**: Removed backward compatibility for old `:set wrap` and `:set nowrap` commands
  - New syntax provides consistency with other settings like `:set number on/off`
  - Updated all tests and documentation to use new syntax exclusively

## [0.32.0] - 2025-01-12

### Added

- **Line Number Toggle**: Added `:set number on/off` commands to toggle line number visibility
  - Line numbers are shown by default (current behavior)
  - Setting persists during the session
  - Provides more screen space for content when hidden

## [0.31.0] - 2025-01-12

### Changed

- **Logging**: Replaced all `println!` and `eprintln!` statements with `tracing` calls throughout the codebase
  - Ensures consistent logging approach as per testing guidelines
  - Improved debugging capabilities with structured logging

## [0.30.0] - 2025-01-12

### Fixed

- **e Command Navigation**: Fixed cursor positioning bug and modified behavior to only target alphanumeric words
  - Fixed incorrect cursor index calculation that prevented the command from working in certain positions
  - Changed behavior to treat punctuation as separators, only stopping at alphanumeric word boundaries
  - Improved predictability when navigating code with punctuation

## [0.29.0] - 2025-01-11

### Added

- **Half Page Navigation**: Complete vim-style half page scrolling commands (issue #78)
  - Ctrl+d: Half page down with proper vim-style virtual column behavior
  - Ctrl+u: Half page up with proper vim-style virtual column behavior
  - DBCS character boundary snapping for multibyte character support
  - Visual selection support during navigation
  - Comprehensive unit testing (4 new tests)

### Technical

- Enhanced PaneManager with `move_cursor_half_page_down()` and `move_cursor_half_page_up()` methods using `div_ceil(2)` for proper half-page calculation
- Added HalfPageDownCommand and HalfPageUpCommand with strict modifier key validation
- Updated AppController to handle HalfPageDown/HalfPageUp movement directions
- Extended CursorManager with wrapper methods for consistent API patterns

## [0.28.0] - 2025-01-12

### Refactored

- **Line Number State Management**: Moved line number width calculation from ViewModel to PaneState, eliminating feature envy and improving architectural consistency (issue #59)
- **Multiline Response Display**: Fixed multiline responses being incorrectly rendered as single lines by removing aggressive newline flattening (issue #115)

### Technical

- Enhanced PaneState with cached line number width management and automatic updates on content changes
- Added PaneManager methods for clean line number width access: `get_line_number_width()` and `get_current_line_number_width()`
- Improved response content handling to preserve original line structure while maintaining single-line response compatibility
- Enhanced test coverage with comprehensive scenarios for both single-line and multiline response handling
- All architectural improvements maintain full backward compatibility while significantly improving code quality

## [0.24.1] - 2025-07-30

### Fixed

- **Visual Mode on Wrapped Lines**: Fixed visual mode selection highlighting not working properly over wrapped line boundaries (issue #31)
- **Cursor Navigation Issues**: Resolved cursor lag and disappearing during Ctrl+F/D page scrolling operations
- **Boundary Condition Bug**: Fixed 'l' key not working when cursor was at column 1 on wrapped text continuation lines
- **Horizontal Scrolling**: Prevented unwanted horizontal scrolling during cursor navigation on wrapped text

### Refactored

- **Feature Envy Elimination**: Moved cursor management business logic from ViewModel to PaneState for better separation of concerns
- **Coordination Structs**: Added CursorMoveResult and ScrollAdjustResult structs to maintain clean architecture
- **Display Position Logic**: Simplified cursor positioning logic and improved logical/display coordinate conversion
- **Visual Selection Updates**: Enhanced visual mode highlighting to properly update during page scrolling operations

### Technical

- Enhanced boundary condition handling in `logical_to_display_position` and `display_to_logical_position` functions
- Improved cursor synchronization between logical and display coordinates
- Added comprehensive debug logging for cursor movement troubleshooting
- All changes validated through extensive manual testing and user feedback

## [0.24.0] - 2025-07-30

### Added

- **Delete Key Empty Line Support**: Enhanced Delete key functionality to support empty line deletion in insert mode. The Delete key now handles three scenarios: (1) when cursor is on an empty line, it deletes the current line and moves cursor to end of previous line; (2) when cursor is at end of a line followed by an empty line, it removes the empty line; (3) when cursor is at end of a line followed by content, it joins the lines as before.

### Technical

- Enhanced `delete_char_after_cursor()` method in buffer_operations.rs with comprehensive empty line detection and handling
- Reordered condition logic to prioritize empty line detection over next-line-exists check
- Added 7 comprehensive unit tests covering all Delete key scenarios including edge cases
- Maintains backward compatibility with existing character deletion behavior
- All changes validated through unit tests and pre-commit checks

## [0.23.0] - 2025-07-29

### Fixed

- **Scrolling Panic Crash**: Fixed critical integer underflow panic when scrolling down in request pane using 'j' key in normal mode. The crash occurred when `start_line` exceeded `request_height` during rapid scrolling, causing "attempt to subtract with overflow" errors.
- **Text Rendering After Scroll**: Fixed issue where characters typed in insert mode after scrolling wouldn't display immediately. The problem was that `PartialPaneRedrawRequired` used absolute display coordinates instead of viewport-relative coordinates.

### Technical

- Applied saturating arithmetic using `saturating_sub()` in terminal_renderer.rs and cursor_manager.rs to prevent integer underflow panics
- Fixed viewport coordinate system mismatch by adjusting `PartialPaneRedrawRequired` coordinates relative to scroll offset
- Enhanced buffer operations with proper coordinate transformation between display cache and terminal renderer
- Added comprehensive error handling for terminal dimension calculations and content width bounds checking

## [0.22.0] - 2025-07-29

### Added

- **Enhanced Backspace Behavior**: Improved backspace functionality for blank lines in insert mode. When backspace is pressed on a blank line (empty line), it now deletes the entire line and moves the cursor to the end of the previous line, providing more intuitive editing experience.

### Technical

- Enhanced `delete_char_before_cursor()` method in buffer_operations.rs with blank line detection using `line_length() == 0`
- Added comprehensive unit tests covering blank line deletion, consecutive blank lines, and cursor positioning
- Refactored pane references to use `current_pane` variable instead of hardcoded `Pane::Request` for better abstraction
- Preserved existing line joining behavior for non-blank lines
- All changes validated through 7 comprehensive unit tests and integration test suite

## [0.21.1] - 2025-07-29

### Fixed

- **Line Navigation Bug**: Fixed `:number` line navigation commands (like `:58`) not working in any pane. Added missing `CursorMoveRequested` event handler in app controller's ex command processing that was causing events to fall through unhandled.
- **Page Scrolling Buffer Erasure**: Fixed Ctrl+f causing buffer content to appear erased in request pane when scrolling beyond actual content bounds. Added bounds checking to prevent scrolling past display cache limits.
- **Half-Page Scrolling Issues**: Fixed Ctrl+d endless scrolling and line number corruption by applying same bounds checking pattern used for full page scrolling.

### Technical

- Enhanced ex command event handling in app_controller.rs with proper `MovementDirection::LineNumber` support
- Added bounds checking using `display_cache.display_line_count().saturating_sub(page_size).max(0)` in rendering_coordinator.rs
- Improved cursor-scroll synchronization to prevent buffer/display state inconsistencies
- All fixes validated through debug log analysis and comprehensive testing

## [0.17.1] - 2025-07-28

### Fixed

- **G Command Shift Key Support**: Fixed `G` command to properly handle `Shift+g` key combination. Command now responds to uppercase G, lowercase g with SHIFT modifier, and uppercase G with SHIFT modifier to ensure compatibility across different terminals.
- **Dynamic Line Number Width**: Fixed line number column width calculation to dynamically adjust based on document size. Previously hardcoded to 3 characters, now expands as needed (e.g., 4 characters for documents with 1000+ lines like line 1547) to prevent cursor positioning issues when jumping between documents of different sizes.
- **Cursor Positioning**: Resolved bug where cursor would appear in invalid positions when using G command to jump from small to large documents due to inconsistent line number column width.

### Technical

- Added `MIN_LINE_NUMBER_WIDTH` constant to replace magic number 3
- Enhanced GoToBottomCommand with comprehensive modifier key handling
- Improved line number width calculation in DisplayManager to be content-aware
- Added comprehensive unit tests for all Shift key combinations

## [0.17.0] - 2025-07-28

### Added

- **New Navigation Command**: Implemented `G` command to go to the bottom of the current pane
- **Document End Navigation**: Added ability to jump to the last line of the document using capital G

### Implementation Details

- Created GoToBottomCommand struct following same pattern as gg command
- Positions cursor at beginning of last line (column 0) following Vim behavior
- Uses same text processing approach as test framework for consistency
- Added comprehensive unit tests covering all edge cases
- Integration test validation ensuring proper cursor positioning

### Technical

- Maintains full compatibility with existing navigation commands
- Leverages existing DocumentEnd movement infrastructure
- Fixed line counting consistency between implementation and test framework

## [0.16.0] - 2025-07-28

### Added

- **New Navigation Command**: Implemented `gg` command to go to the top of the current pane
- **G Prefix Mode**: Added EditorMode::GPrefix to handle Vim-style 'g' prefix commands
- **Document Navigation**: Added move_cursor_to_document_start() and move_cursor_to_document_end() methods

### Implementation Details

- Two new command structs: EnterGPrefixCommand and GoToTopCommand
- Enhanced MovementDirection enum with DocumentStart and DocumentEnd variants
- Updated controller to handle document-level cursor movements
- Comprehensive unit tests covering all new functionality
- Integration test validation ensuring proper behavior

### Technical

- Added GPrefix support to terminal renderer for proper cursor display
- Updated test framework to handle new movement directions
- Maintains full compatibility with existing navigation commands

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
