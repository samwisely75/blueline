# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.23.0] - 2025-07-29

### Fixed
- Fixed panic crash when scrolling down past bottom of request pane with 'j' key in normal mode
- Fixed text rendering issue where typed characters weren't displayed immediately after scrolling in insert mode
- Added saturating arithmetic to prevent integer underflow in terminal renderer and cursor manager
- Converted display coordinates to viewport-relative coordinates for proper partial redraw events

## [0.22.0] - 2025-07-29

### Added
- **Enhanced Backspace Behavior**: Improved backspace functionality for blank lines in insert mode. When backspace is pressed on a blank line (empty line), it now deletes the entire line and moves the cursor to the end of the previous line, providing more intuitive editing experience.

### Technical
- Enhanced `delete_char_before_cursor()` method in buffer_operations.rs with blank line detection using `line_length() == 0`
- Added comprehensive unit tests covering blank line deletion, consecutive blank lines, and cursor positioning
- Refactored pane references to use `current_pane` variable instead of hardcoded `Pane::Request` for better abstraction
- Preserved existing line joining behavior for non-blank lines
- All changes validated through 7 comprehensive unit tests and integration test suite

## [0.21.0] - 2025-07-28

### Added
- Support `b` command to skip to the previous word in the request/response pane
- Support `e` command to skip to the end of the current/next word in the request/response pane
- Support `0` and `Home` keys to go to the beginning of the current line in the request/response pane
- Support `$` and `End` keys to go to the end of the current line in the request/response pane
- All word and line navigation commands follow Vim behavior and work in both request and response panes

### Changed
- Line navigation commands (`0`, `$`, `Home`, `End`) now work with logical lines instead of display lines, matching Vim behavior
- Improved `e` command logic to properly handle word boundaries and cursor positioning

## [0.20.0] - 2025-07-28

### Added
- Support `w` command to skip to the next word in the request/response pane
- Word navigation follows Vim behavior: moves to beginning of next word, skips punctuation and whitespace
- Command works in both request and response panes in Normal mode

## [0.19.0] - 2025-07-28

### Added
- Show "Executing..." message with yellow bullet in status bar during request processing
- Request execution status tracking to prevent multiple simultaneous requests

### Changed
- Keep cursor visible in request pane with underline shape during Command mode
- Show I-beam cursor in status bar command line for better command editing experience
- Dim status bar when not in Command mode to reduce visual clutter
- Command buffer automatically clears when exiting Command mode (Escape or Ctrl+C)

## [0.18.0] - 2025-07-28

### Added
- Support `Ctrl + b` to scroll up one page in the request/response pane (Vim-style page scrolling)
- Support `Ctrl + d` to scroll down half a page in the request/response pane (Vim-style half-page scrolling)
- Support `Ctrl + u` to scroll up half a page in the request/response pane (Vim-style half-page scrolling)
- Comprehensive unit tests for all three new scroll commands (12 new tests)
- Added HalfPageDown and HalfPageUp variants to MovementDirection enum

## [0.17.2] - 2025-07-28

### Added
- Support `Ctrl + f` to scroll down one page in the request/response pane with context preservation (Vim-style)

### Fixed
- Fixed cursor positioning bug where logical cursor wasn't updated after page scrolling operations
- Fixed display cache invalidation bug when toggling word wrap mode that caused view to reset while cursor position indicator remained incorrect
- Fixed page down scrolling with horizontal scroll offset causing cursor to appear off-screen
- Fixed horizontal scrolling cursor position bug where position indicator wouldn't update during Shift+Arrow scrolling

## [0.17.1] - 2025-07-28

### Fixed
- Fixed G command to properly handle Shift+g key combinations across different terminals
- Fixed dynamic line number column width calculation to prevent cursor positioning issues

## [0.17.0] - 2025-07-28

### Added
- Support `G` to go to the bottom of the current pane
- Support `gg` to go to the top of the current pane
- G prefix mode for two-key command sequences

## [0.16.0] - 2025-07-28

### Added
- Support `gg` to go to the top of the current pane
- G prefix mode for implementing two-key Vim commands

## [0.15.1] - 2025-07-28

### Changed
- Renamed command terminology for clarity and alignment to Vim terminologies
- Renamed movement.rs to navigation.rs and updated all references

## [0.15.0] - 2025-07-28

### Fixed
- Cleaned up unauthorized header examples from feature files
- Removed blank lines between HTTP command line and data lines in test scenarios