Feature: Editor Mode Transitions
  As a developer using blueline
  I want to switch between different editor modes (Normal, Insert, Command)
  So that I can follow vi-style editing patterns for HTTP requests

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  Scenario: Enter insert mode and edit text
    Given the request buffer is empty
    And I am in normal mode
    When I press "i"
    Then I am in insert mode
    And the cursor style changes to a blinking bar
    When I type "GET /api/users"
    Then the text appears in the request buffer
    When I press Escape
    Then I am in normal mode
    And the cursor style changes to a steady block

  Scenario: Cancel command mode
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I press Escape
    Then I am in normal mode
    And the command buffer is cleared

  Scenario: Edit multiline HTTP request
    Given the request buffer is empty
    And I am in normal mode
    When I press "i"
    Then I am in insert mode
    When I type:
      """
      POST /api/users

      {"name": "John Doe"}
      """
    And I press Escape
    Then I am in normal mode
    And the request buffer contains the multiline request
