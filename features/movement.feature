Feature: Cursor Movement Commands
  As a developer using blueline
  I want to navigate through HTTP request text using vim-style cursor movement
  So that I can efficiently position my cursor for editing

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

  Scenario: Navigate response content
    Given I have executed a request that returned a large JSON response from:
      """
      GET /api/users
      """
    And I am in the response pane
    When I use vim navigation keys
    Then I can scroll through the response content
    And line numbers are visible

  Scenario: Scroll up with Ctrl+U
    Given the request buffer contains:
      """
      GET /api/users
      Host: example.com
      Content-Type: application/json

      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      """
    And I am in normal mode
    And the cursor is at line 15
    When I press "Ctrl+U"
    Then the cursor moves up by half a page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Ctrl+D
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Ctrl+D"
    Then the cursor moves down by half a page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Ctrl+F
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Ctrl+F"
    Then the cursor moves down by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll up with Ctrl+B
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 25
    When I press "Ctrl+B"
    Then the cursor moves up by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Page Down key
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Page Down"
    Then the cursor moves down by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll up with Page Up key
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 25
    When I press "Page Up"
    Then the cursor moves up by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode
