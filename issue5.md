## Problem

When wrap mode is enabled and typing in insert mode, the display incorrectly performs horizontal scrolling instead of wrapping to the next line when content exceeds the pane width.

## Test Scenario Setup

- Fresh application state
- Request pane width: 106 display columns
- Wrap mode: ON (enabled)
- Mode: Insert mode
- Content: Single line with single-byte characters

## Expected Behavior (Wrap Mode ON)

When typing in insert mode with wrap enabled:
1. Type 106 single-byte characters - all should be visible on first display line
2. Type 107th character - cursor should move to the beginning of second display line
3. **No horizontal scrolling should occur**
4. All content should remain visible through line wrapping

## Current Behavior (Incorrect)

When typing in insert mode with wrap enabled:
1. Type 106 single-byte characters - correctly fills first display line
2. Type 107th character - **Bug**: Screen scrolls horizontally by one character width to the right
3. **Bug**: Behaves as if in nowrap mode despite wrap being enabled
4. **Bug**: Content is not wrapped to next display line

## Impact

- Wrap mode is non-functional during text input
- Users cannot effectively use wrap mode for long line editing
- Display behavior is inconsistent with wrap mode setting
- Text input experience is confusing and unpredictable

## Technical Analysis

This suggests that:
1. **Insert mode logic** is not checking wrap mode setting during character insertion
2. **Display refresh** defaults to nowrap behavior regardless of wrap setting
3. **Line wrapping logic** is not triggered during live text input
4. **Cursor positioning** follows horizontal scroll logic instead of wrap logic

## Root Causes (Suspected)

1. **Text insertion logic**: Insert mode not consulting wrap mode setting
2. **Display update**: Screen refresh using nowrap logic even when wrap is enabled
3. **Event handling**: Character insertion events not triggering wrap calculations
4. **MVVM coordination**: ViewModel not properly communicating wrap state to View during insertion

## Related Files

This likely involves:
- Insert mode character handling
- Wrap mode configuration and state management
- Display refresh logic during text input
- Cursor positioning after character insertion
- Line wrapping implementation
- Screen coordinate calculations

## Technical Notes

This is a critical wrap mode functionality issue. The text insertion logic needs to check the wrap setting and trigger appropriate display updates (line wrapping vs horizontal scrolling) based on the current mode.

## Related Issues

- #80: Dollar sign positioning with double-byte characters
- #81: Horizontal scrolling with multibyte characters
- #82: Text append broken with double-byte characters and horizontal scrolling
- #83: Wrap mode toggle does not reflow display with double-byte characters
