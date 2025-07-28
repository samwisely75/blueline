# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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