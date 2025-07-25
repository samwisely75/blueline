Feature: Screen Refresh Tracking - Key Events Test
  As a developer
  I want to verify that screen refresh operations are called appropriately for key events
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls appropriate render methods for key events
    Given a REPL controller with mock view renderer
    And the controller has started up
    When I clear the render call history
    And I simulate pressing "h" key (move left)
    Then render_cursor_only should be called once
    And no other render methods should be called
