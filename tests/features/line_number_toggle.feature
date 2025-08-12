Feature: Line Number Toggle
  As a user
  I want to toggle line numbers on and off
  So that I can customize the interface density

  Background:
    Given the application is started with default settings

  Scenario: Line numbers are visible by default
    When the application starts
    Then I should see line number "1" in the request pane
    And the cursor should be positioned after the line number

  Scenario: Hide line numbers with :set number off
    Given I am in Normal mode
    When I enter command mode
    And I type "set number off"
    And I press Enter
    Then I should not see line numbers in the request pane
    And the cursor should be positioned at the start of the line

  Scenario: Show line numbers with :set number on
    Given I am in Normal mode
    And line numbers are hidden
    When I enter command mode
    And I type "set number on"
    And I press Enter
    Then I should see line number "1" in the request pane
    And the cursor should be positioned after the line number

  Scenario: Line number visibility affects both panes
    Given I have executed a request
    When I enter command mode
    And I type "set number off"
    And I press Enter
    Then I should not see line numbers in the request pane
    And I should not see line numbers in the response pane

  Scenario: Content width increases when line numbers are hidden
    Given I have text "This is a test line" in the request buffer
    When I enter command mode
    And I type "set number off"
    And I press Enter
    Then the full width of the terminal should be available for content