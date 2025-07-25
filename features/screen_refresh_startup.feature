Feature: Screen Refresh Tracking - Startup Test
  As a developer
  I want to verify that screen refresh operations are called appropriately on startup
  So that I can ensure proper rendering behavior

  @mock
  Scenario: Controller calls render_full on startup
    Given a REPL controller with mock view renderer
    When the controller starts up
    Then render_full should be called once
    And initialize_terminal should be called once
