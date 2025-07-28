# Cursor Flicker Fix Test
#
# This feature specifically tests the fix for cursor flickering in the request pane
# when cursor movements are performed. The fix ensures proper cursor hiding/showing
# during render_cursor_only operations.

Feature: Cursor Flicker Fix
  As a user
  I want cursor movements in the request pane to be smooth without flickering
  So that I can navigate efficiently without visual distractions

  @mock
  Scenario: Cursor-only updates should hide and show cursor properly
    Given a REPL controller with mock view renderer
    And the controller has started up
    And I am in the request pane with content:
      """
      GET /api/users
      {"name": "test"}
      """
    When I clear the render call history
    And I simulate rapid cursor movements with "h", "l", "j", "k"
    Then render_cursor_only should be called multiple times
    And each cursor update should maintain proper cursor visibility
    And no flickering should occur during rapid movements

  Scenario: Request pane cursor movements should be smooth in real usage
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode
    And the request buffer contains:
      """
      GET /api/users
      {"name": "John Doe", "email": "john@example.com"}
      """
    When I navigate using vim keys "h", "j", "k", "l" rapidly
    Then cursor movements should be smooth without visual artifacts
    And the cursor should remain visible at all times
    And no screen flickering should occur
