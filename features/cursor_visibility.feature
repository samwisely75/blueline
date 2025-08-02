# Cursor Visibility in Command Mode Test
#
# This feature tests the requirement that cursor should be hidden when entering
# command mode and restored when switching back to normal mode.

Feature: Cursor Visibility Management
  As a user
  I want the cursor to be hidden when I enter command mode
  So that the command line interface is clean and focused

  @mock
  Scenario: Cursor should be hidden in command mode and restored in normal mode
    Given a REPL controller with mock view renderer
    And the controller has started up
    And I am in normal mode
    When I clear the render call history
    And I simulate pressing ":" to enter command mode
    Then render_full should be called once
    And the cursor should be hidden in command mode
    When I simulate pressing Escape to return to normal mode
    Then render_full should be called again
    And the cursor should be visible in normal mode

  Scenario: Real cursor visibility transitions
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode
    And the cursor is visible
    When I press ":"
    Then I am in command mode
    And the cursor should be hidden
    When I press Escape
    Then I am in normal mode
    And the cursor should be visible again

  Scenario: Cursor visibility with different mode transitions
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode
    When I press "i"
    Then I am in insert mode
    And the cursor should be visible with blinking bar style
    When I press Escape
    Then I am in normal mode
    And the cursor should be visible with steady block style
    When I press ":"
    Then I am in command mode
    And the cursor should be hidden
    When I press Escape
    Then I am in normal mode
    And the cursor should be visible with steady block style
