Feature: Screen Refresh Tracking - Mode Changes Test
  As a developer
  I want to verify that screen refresh operations are called appropriately for mode changes
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls render_full for mode changes
    Given a REPL controller with mock view renderer
    And the controller has started up
    When I clear the render call history
    And I simulate pressing "i" to enter insert mode
    Then render_full should be called once
    And the state snapshot should show Insert mode
