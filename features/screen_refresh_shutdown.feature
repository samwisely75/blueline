Feature: Screen Refresh Tracking - Shutdown Test
  As a developer
  I want to verify that screen refresh operations are called appropriately on shutdown
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls cleanup_terminal on shutdown
    Given a REPL controller with mock view renderer
    When the controller shuts down
    Then cleanup_terminal should be called once
