Feature: Blueline HTTP Client REPL
  As a developer
  I want to interact with HTTP APIs using a vim-style terminal interface
  So that I can efficiently test and debug web services

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  Scenario: Basic vim navigation
    Given the request buffer contains:
      """
      POST /api/users

      {"name": "John Doe"}
      """
    And I am in normal mode
    When I press "h"
    Then the cursor moves left
    When I press "l"
    Then the cursor moves right
    When I press "j"
    Then the cursor moves down
    When I press "k"
    Then the cursor moves up
    And I am still in normal mode

  Scenario: Line navigation
    Given the request buffer contains:
      """
      POST /api/users

      {"name": "John Doe"}
      """
    And I am in normal mode
    When I press "0"
    Then the cursor moves to the beginning of the line
    And I am still in normal mode
    When I press "$"
    Then the cursor moves to the end of the line
    And I am still in normal mode

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

  Scenario: Switch between panes
    Given there is a response in the response pane
    And I am in normal mode
    When I press "Ctrl+W"
    And I press "j"
    Then I am in the response pane
    And I am in normal mode
    When I press "Ctrl+W"
    And I press "k"
    Then I am in the request pane
    And I am in normal mode

  Scenario: Execute HTTP request
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "x"
    And I press Enter
    Then the HTTP request is executed
    And I am in normal mode
    And the response appears in the response pane
    And I can see the status code

  Scenario: Quit from request pane
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "q"
    And I press Enter
    Then the application exits

  Scenario: Close response pane
    Given there is a response in the response pane from:
      """
      GET /api/users
      """
    And I am in the response pane
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "q"
    And I press Enter
    Then the response pane closes
    And I am in the request pane
    And I am in normal mode
    And the request pane is maximized

  Scenario: Force quit from any pane
    Given I am in the request pane with the buffer containing:
      """
      POST /api/users

      {"name": "John"}
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "q!"
    And I press Enter
    Then the application exits without saving

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

  Scenario: Handle unknown command
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "unknown"
    And I press Enter
    Then I see an error message "Unknown command: unknown"
    And I am in normal mode

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
    When I press ":"
    Then I am in command mode
    When I type "x"
    And I press Enter
    Then the POST request is executed with the JSON body
    And I am in normal mode

  Scenario: Navigate response content
    Given I have executed a request that returned a large JSON response from:
      """
      GET /api/users
      """
    And I am in the response pane
    When I use vim navigation keys
    Then I can scroll through the response content
    And line numbers are visible

  Scenario: Start with verbose mode
    Given blueline is started with "-v" flag
    When I execute a request:
      """
      GET /api/status
      """
    Then I see detailed request information
    And I see response headers
    And I see timing information

  Scenario: Use custom profile
    Given blueline is started with "-p staging" flag
    When I execute "GET /api/status"
    Then the request uses the staging profile configuration
    And the base URL is taken from the staging profile
