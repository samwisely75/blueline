Feature: Visual Mode Text Selection
  As a developer using blueline
  I want to select text using vim-style visual mode
  So that I can easily select and manipulate text in HTTP requests

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  Scenario: Enter and exit visual mode
    Given the request buffer contains:
      """
      GET /api/users
      """
    And I am in normal mode
    And the cursor is at line 0 column 0
    When I press "v"
    Then I am in visual mode
    And the cursor style remains as a block cursor
    And the status bar shows "VISUAL" mode
    When I press Escape
    Then I am in normal mode
    And the status bar shows "NORMAL" mode
    And no text is selected

  Scenario: Select text within a single line
    Given the request buffer contains:
      """
      GET /api/users
      """
    And I am in normal mode
    And the cursor is at line 0 column 4
    When I press "v"
    Then I am in visual mode
    When I press "l" 3 times
    Then the text "api" is selected
    And the selected text is highlighted with blue background and inverse colors
    When I press Escape
    Then I am in normal mode
    And no text is selected

  Scenario: Select text across multiple lines
    Given the request buffer contains:
      """
      POST /api/users
      {"name": "John"}
      """
    And I am in normal mode
    And the cursor is at line 1 column 8
    When I press "v"
    Then I am in visual mode
    When I press "j" 2 times
    And I press "l" 5 times
    Then multiple lines are selected
    And the selected text spans from line 1 to line 3
    And all selected text is highlighted with blue background and inverse colors
    When I press Escape
    Then I am in normal mode
    And no text is selected

  Scenario: Visual mode navigation commands work
    Given the request buffer contains:
      """
      GET /api/users HTTP/1.1
      Host: api.example.com
      Authorization: Bearer token
      Content-Type: application/json
      """
    And I am in normal mode
    And the cursor is at line 0 column 0
    When I press "v"
    Then I am in visual mode
    # Test word navigation
    When I press "w"
    Then the cursor moves to the next word
    And text is selected from the start position to current cursor
    # Test line navigation
    When I press "$"
    Then the cursor moves to the end of the line
    And text is selected from the original start to end of line
    # Test line jumping
    When I press "j"
    Then the cursor moves down one line
    And text is selected across multiple lines
    When I press Escape
    Then I am in normal mode

  Scenario: Visual mode with vim movement commands
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/orders
      PUT /api/settings
      DELETE /api/cache
      """
    And I am in normal mode
    And the cursor is at line 1 column 0
    When I press "v"
    Then I am in visual mode
    # Test various vim movements
    When I press "w" 2 times
    Then text is selected
    When I press "b"
    Then the selection is adjusted backward 
    When I press "e"
    Then the selection extends to end of word
    When I press "G"
    Then the selection extends to the last line
    When I press Escape
    Then I am in normal mode

  Scenario: Visual mode persists during navigation
    Given the request buffer contains:
      """
      Line one
      Line two  
      Line three
      """
    And I am in normal mode
    And the cursor is at line 0 column 0
    When I press "v"
    Then I am in visual mode
    When I press "h"
    Then I remain in visual mode
    When I press "j"
    Then I remain in visual mode
    When I press "k"
    Then I remain in visual mode
    When I press "l"
    Then I remain in visual mode
    And text selection is updated with each movement
    When I press Escape
    Then I am in normal mode

  Scenario: Visual mode works in response pane
    Given there is a response in the response pane from:
      """
      {
        "users": [
          {"id": 1, "name": "Alice"}, 
          {"id": 2, "name": "Bob"}
        ]
      }
      """
    And I am in the response pane
    And I am in normal mode
    And the cursor is at line 1 column 2
    When I press "v"
    Then I am in visual mode
    When I press "w" 2 times
    And I press "j"
    Then text is selected in the response pane
    And the selected text is highlighted with blue background and inverse colors
    When I press Escape
    Then I am in normal mode
    And no text is selected

  Scenario: Visual mode selection does not cross panes
    Given the request buffer contains:
      """
      GET /api/test
      """
    And there is a response in the response pane from:
      """
      {"result": "success"}
      """
    And I am in the request pane
    And I am in normal mode
    When I press "v"
    Then I am in visual mode
    When I press "l" 3 times
    Then text is selected in the request pane
    When I press Tab
    Then I am in the response pane
    But the visual selection remains in the request pane
    And no new selection starts in the response pane
    When I press Escape
    Then I am in normal mode
    And no text is selected in either pane

  Scenario: Visual mode with scrolling commands
    Given the request buffer contains a large text with 50 lines
    And I am in normal mode
    And the cursor is at line 5 column 0
    When I press "v"
    Then I am in visual mode
    When I press "Ctrl+f"
    Then the selection extends down by a full page
    And text spanning multiple pages is selected
    When I press "Ctrl+b"
    Then the selection is adjusted by scrolling up
    When I press Escape
    Then I am in normal mode

  Scenario: Visual mode selection highlights correctly
    Given the request buffer contains:
      """
      GET /api/users?filter=active
      """
    And I am in normal mode
    And the cursor is at line 0 column 4
    When I press "v"
    Then I am in visual mode
    When I press "l" 10 times
    Then the text "/api/users" is selected
    And each selected character has blue background color
    And each selected character has inverted foreground color
    And non-selected characters remain with normal styling
    When I press Escape
    Then I am in normal mode
    And all text returns to normal styling