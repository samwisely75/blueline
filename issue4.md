## Problem

When toggling wrap mode on with `:set wrap` command, the display does not properly reflow content that contains double-byte characters extending beyond the pane width. The screen remains in nowrap display mode despite the setting change.

## Test Scenario Setup

- Request pane with wrap mode initially OFF
- Line content: 55 double-byte characters (110 display columns)
- Pane width: 112 bytes (less than 110 display columns)
- Current view: Horizontally scrolled to show characters beyond pane width

## Expected Behavior

When executing `:set wrap`:
1. Screen should reload/refresh
2. Long line should reflow to multiple display lines
3. Characters 54-55 should wrap to the second display line
4. Horizontal scroll should be disabled
5. All content should be visible without horizontal scrolling

## Current Behavior

When executing `:set wrap`:
1. Command executes without error
2. **Bug**: Screen does not reload/reflow the content
3. **Bug**: Display remains in nowrap mode appearance
4. **Bug**: Characters 54-55 remain off-screen or horizontally scrolled
5. **Bug**: Line does not wrap to multiple display lines

## Impact

- Wrap mode toggle is non-functional with multibyte content
- Users cannot switch between wrap modes effectively
- Long lines with double-byte characters remain inaccessible
- Display state becomes inconsistent with actual wrap setting

## Related Functionality

This likely affects:
- `:set nowrap` command (reverse direction)
- Initial rendering with wrap mode on
- Other dynamic display setting changes

## Root Causes (Suspected)

1. **Display refresh logic**: Screen reflow not triggered after wrap setting change
2. **Character width calculation**: Wrap logic not accounting for double-byte character display width
3. **Event handling**: Setting change not properly propagating to display layer
4. **MVVM state sync**: ViewModel not updating View after configuration change

## Related Files

This likely involves:
- Command execution for `:set` commands
- Display configuration management
- Screen refresh/reflow logic
- Line wrapping implementation
- Character width calculation
- MVVM state synchronization

## Technical Notes

This is part of the broader i18n support issues in blueline. The wrap mode toggle should trigger a complete display refresh that accounts for multibyte character widths during line reflow calculations.

## Related Issues

- #80: Dollar sign positioning with double-byte characters
- #81: Horizontal scrolling with multibyte characters
- #82: Text append broken with double-byte characters and horizontal scrolling
