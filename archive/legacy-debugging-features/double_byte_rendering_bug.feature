Feature: Double-byte Character Rendering Bug
  As a developer
  I want to identify the root cause of rendering failures with double-byte characters
  So that the response pane displays content correctly and the request pane doesn't black out

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  @bug @double_byte @rendering
  Scenario: Reproduce double-byte character rendering failure
    Given the request buffer is empty
    # Start => i => GET _search => Esc => Enter
    When I press "i"
    Then I am in insert mode
    When I type "GET _search"  
    And I press Escape
    Then I am in normal mode
    When I press Enter
    # At this point the bug manifests:
    # - Response pane comes up with nothing
    # - Request pane gets blacked out
    Then I capture the terminal state for debugging
    And the response pane should display content
    And the request pane should not be blacked out
    And the terminal should show both panes correctly

  @bug @double_byte @vte_debugging
  Scenario: Detailed VTE analysis of rendering failure
    Given the request buffer is empty
    And I clear the terminal capture
    # Execute the problematic sequence with detailed VTE capture
    When I press "i"
    And I type "GET _search"
    And I press Escape
    And I press Enter
    # Capture detailed terminal state
    Then I capture the full terminal grid state
    And I verify the request pane visual content
    And I verify the response pane visual content
    And I check for rendering statistics anomalies
    And I verify cursor position correctness
    # Specific checks for the bug symptoms
    And the response pane should not be completely empty
    And the request pane should not be completely black
    And both panes should have visible borders
    And the status line should be visible

  @bug @double_byte @step_by_step
  Scenario: Step-by-step analysis of rendering breakdown
    Given the request buffer is empty
    # Test each step individually to isolate the problem
    When I press "i"
    Then the terminal state should be valid
    When I type "GET _search"
    Then the terminal state should be valid  
    And the request pane should be visible
    And the request pane should show "GET _search"
    And the cursor should be positioned correctly
    When I press Escape
    Then the terminal state should be valid
    And I am in normal mode
    And the request pane should still show "GET _search"
    When I press Enter
    # This is where the bug likely occurs
    Then I capture detailed rendering statistics
    And the terminal state should be valid
    And both panes should be properly rendered
    And the response pane should show HTTP response content
    And the request pane should still show "GET _search"