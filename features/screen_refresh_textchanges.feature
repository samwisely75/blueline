Feature: Screen Refresh Tracking - Text Changes Test
  As a developer
  I want to verify that screen refresh operations are called appropriately for text changes
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls render_content_update for text changes
    Given a REPL controller with mock view renderer
    And the controller has started up
    And I am in insert mode
    When I clear the render call history
    And I simulate typing "GET /api/users"
    Then render_content_update should be called multiple times
    And the state snapshots should show content changes
