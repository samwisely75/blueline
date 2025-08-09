Feature: Cursor Visibility Management
  As a user
  I want the cursor to be hidden when I enter command mode
  So that the command line interface is clean and focused

  Background:
    Given I am in Normal mode

  Scenario: Real cursor visibility transitions
    Given the cursor is visible
    When I enter command mode
    Then I should be in Command mode
    And the cursor should be hidden
    When I press Escape
    Then I should be in Normal mode
    And the cursor should be visible

  Scenario: Cursor visibility with different mode transitions
    When I press "i"
    Then I should be in Insert mode
    And the cursor should be visible with blinking bar style
    When I press Escape
    Then I should be in Normal mode
    And the cursor should be visible with steady block style
    When I enter command mode
    Then I should be in Command mode
    And the cursor should be hidden
    When I press Escape
    Then I should be in Normal mode
    And the cursor should be visible with steady block style