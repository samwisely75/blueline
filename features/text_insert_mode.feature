Feature: Text Insert Mode Operations
  As a developer using blueline
  I want to insert text efficiently in insert mode
  So that I can compose HTTP requests quickly

  Background:
    Given the scenario state is reset
    And blueline is running with default profile
    And I am in the request pane

  # === BASIC INSERT MODE SCENARIOS ===
  
  Scenario: Enter insert mode and type basic text
    Given I am in normal mode
    When I press "i"
    And I type "GET /api/users"
    And I press Escape
    Then the screen should not be blank
    And I should see "GET /api/users" in the request pane
    And the cursor should be positioned correctly

  Scenario: Insert character in middle of wrapped line
    Given I am in insert mode
    And the request buffer contains "GET /api/very-long-endpoint-that-exceeds-terminal-width-and-wraps-to-next-line"
    And the text wraps to a second line due to terminal width
    And the cursor is positioned in the middle of the line after "very-"
    When I type "x"
    Then the character "x" is inserted at the cursor position
    And the text becomes "GET /api/very-xlong-endpoint-that-exceeds-terminal-width-and-wraps-to-next-line"
    And the wrapped text in the second line expands by one character
    And the cursor moves forward one position

  Scenario: Insert newline with Enter key
    Given I am in insert mode
    And the request buffer contains "GET /api/users"
    And the cursor is at the end of the line
    When I press Enter to create a new line
    Then the request buffer contains the multiline request
    And the cursor is on the new line

  Scenario: Multi-line editing with Enter
    Given I am in insert mode
    And the request buffer contains "GET /api/users"
    When I press Enter to create a new line
    And I type "Content-Type: application/json"
    Then the text appears in the request buffer
    And the cursor position advances with each character

  Scenario: Insert multiline request with newlines
    Given I am in insert mode
    And the request buffer is empty
    When I type:
      """
      GET /api/users
      Accept: application/json
      Content-Type: application/json

      {"name": "John"}
      """
    Then the text appears in the request buffer
    And the request buffer contains the multiline request

  Scenario: Insert text with quotes and escape sequences
    Given I am in insert mode
    And the request buffer is empty
    When I type "Content-Type: \"application/json\" with escaped quotes"
    Then the text appears in the request buffer
    And the quotes are properly handled

  Scenario: Handle problematic characters that might cause parsing errors  
    Given I am in insert mode
    And the request buffer is empty
    When I type "Special chars: @#$%^&*()_+-=[]{}|;':\",./<>?"
    Then the text appears in the request buffer
    And all special characters are preserved correctly
    And the screen should not be blank