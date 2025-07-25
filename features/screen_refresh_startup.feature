# Screen Refresh Tracking - Individual Scenario Files
#
# NOTE: Each screen refresh test scenario is in its own feature file to prevent
# thread_local storage interference between BDD scenarios. Originally, all scenarios
# were in one file, but Cucumber runs them sequentially in the same thread, causing
# MockViewRenderer state to persist and accumulate across scenarios.
#
# See tests/integration_tests.rs and tests/common/steps.rs for detailed explanation.

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
