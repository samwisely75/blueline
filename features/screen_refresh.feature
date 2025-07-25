Feature: Screen Refresh Tracking
  As a developer
  I want to verify that screen refresh operations are called appropriately
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls render_full on startup
    Given a REPL controller with mock view renderer
    When the controller starts up
    Then render_full should be called once
    And initialize_terminal should be called once

  @mock  
  Scenario: Controller calls appropriate render methods for key events
    Given a REPL controller with mock view renderer
    And the controller has started up
    When I clear the render call history
    And I simulate pressing "h" key (move left)
    Then render_cursor_only should be called once
    And no other render methods should be called

  @mock
  Scenario: Controller calls render_content_update for text changes
    Given a REPL controller with mock view renderer
    And the controller has started up
    And I am in insert mode
    When I clear the render call history
    And I simulate typing "GET /api/users"
    Then render_content_update should be called multiple times
    And the state snapshots should show content changes

  @mock
  Scenario: Controller calls render_full for mode changes
    Given a REPL controller with mock view renderer
    And the controller has started up
    When I clear the render call history
    And I simulate pressing "i" to enter insert mode
    Then render_full should be called once
    And the state snapshot should show Insert mode

  @mock
  Scenario: Controller calls cleanup_terminal on shutdown
    Given a REPL controller with mock view renderer
    When the controller shuts down
    Then cleanup_terminal should be called once
