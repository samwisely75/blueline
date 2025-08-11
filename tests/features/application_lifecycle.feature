Feature: Application Lifecycle
  As a developer using blueline
  I want to start and terminate the application cleanly
  So that I can use the HTTP client reliably in different scenarios

  Background:
    Given the application is started with default settings

  Scenario: Application startup with default settings
    When the application starts
    Then I should be in Normal mode
    And the request pane should show line number "1" in column 3
    And the request pane should show "~" for empty lines
    And there should be a blinking block cursor at column 4
    And the status bar should show "REQUEST | 1:1" aligned to the right
    And there should be no response pane visible

  Scenario: Quit application with colon command
    Given I am in Normal mode
    When I enter command mode
    Then I should be in Command mode
    And I type "q"
    And I press Enter
    Then the application should terminate cleanly

  Scenario: Force quit application
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "some unsaved content"
    When I press Escape
    Then I should be in Normal mode
    When I enter command mode
    And I type "q!"
    And I press Enter
    Then the application should terminate without saving
