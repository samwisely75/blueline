Feature: Improved Wrap Command Syntax
  As a user
  I want to use consistent on/off syntax for wrap settings
  So that commands are easier to remember and use

  Background:
    Given the application is started with default settings

  Scenario: Enable wrap with new syntax
    Given I am in Normal mode
    When I enter command mode
    And I type "set wrap on"
    And I press Enter
    Then wrap mode should be enabled

  Scenario: Disable wrap with new syntax
    Given I am in Normal mode
    And wrap mode is enabled
    When I enter command mode
    And I type "set wrap off"
    And I press Enter
    Then wrap mode should be disabled


  Scenario: Toggle wrap state multiple times
    Given I am in Normal mode
    When I enter command mode
    And I type "set wrap on"
    And I press Enter
    Then wrap mode should be enabled
    When I enter command mode
    And I type "set wrap off"
    And I press Enter
    Then wrap mode should be disabled
    When I enter command mode
    And I type "set wrap on"
    And I press Enter
    Then wrap mode should be enabled