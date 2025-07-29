Feature: Text Editing Commands
  As a developer using blueline
  I want to edit HTTP request text in insert mode
  So that I can compose and modify requests efficiently

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in insert mode

  Scenario: Insert regular text characters
    Given the request buffer is empty
    And I am in insert mode
    When I type "GET /api/users"
    Then the text appears in the request buffer
    And the cursor position advances with each character

  Scenario: Insert special characters that might cause issues
    Given the request buffer is empty
    And I am in insert mode
    When I type "GET /api/users?email=user@example.com"
    Then the text appears in the request buffer
    And the at-sign "@" is properly inserted

  Scenario: Insert backtick characters
    Given the request buffer is empty
    And I am in insert mode
    When I type "Authorization: Bearer `token123`"
    Then the text appears in the request buffer
    And the backticks "`" are properly inserted

  Scenario: Insert newline with Enter key
    Given the request buffer contains "GET /api/users"
    And I am in insert mode
    And the cursor is at the end of the line
    When I press Enter
    Then a new line is created
    And the cursor moves to the beginning of the new line
    And the previous line remains unchanged

  Scenario: Insert multiline request with newlines
    Given the request buffer is empty
    And I am in insert mode
    When I type:
      """
      POST /api/users
      Content-Type: application/json

      {"name": "John"}
      """
    Then the request buffer contains multiple lines
    And empty lines are preserved
    And JSON formatting is maintained

  Scenario: Delete character with Backspace
    Given the request buffer contains "GET /api/userss"
    And I am in insert mode
    And the cursor is at the end of the line
    When I press Backspace
    Then the last "s" character is deleted
    And the cursor moves back one position
    And the text becomes "GET /api/users"

  Scenario: Delete character in middle of line
    Given the request buffer contains "GET /appi/users"
    And I am in insert mode
    And the cursor is positioned after the extra "i"
    When I press Backspace
    Then the extra "i" is deleted
    And the text becomes "GET /api/users"

  Scenario: Join lines with Backspace at line start
    Given the request buffer contains:
      """
      GET /api/users
      Second line
      """
    And I am in insert mode
    And the cursor is at the beginning of the second line
    When I press Backspace
    Then the lines are joined together
    And the text becomes "GET /api/usersSecond line"
    And the cursor position is correct

  Scenario: Backspace at beginning of first line should not delete
    Given the request buffer contains "GET /api/users"
    And I am in insert mode
    And the cursor is at the beginning of the first line
    When I press Backspace
    Then no character is deleted
    And the text remains "GET /api/users"
    And the cursor stays at the beginning

  Scenario: Backspace on blank line deletes entire line
    Given the request buffer contains:
      """
      GET /api/users

      {"name": "John"}
      """
    And I am in insert mode
    And the cursor is on the blank line (line 2)
    When I press Backspace
    Then the blank line is deleted
    And the cursor moves to the end of the previous line
    And the text becomes:
      """
      GET /api/users
      {"name": "John"}
      """

  Scenario: Backspace on consecutive blank lines deletes current blank line
    Given the request buffer contains:
      """
      GET /api/users


      {"name": "John"}
      """
    And I am in insert mode
    And the cursor is on the second blank line (line 3)
    When I press Backspace
    Then only the current blank line is deleted
    And the cursor moves to the end of the previous line (first blank line)
    And the text becomes:
      """
      GET /api/users

      {"name": "John"}
      """

  Scenario: Handle problematic characters that might cause parsing errors
    Given the request buffer is empty
    And I am in insert mode
    When I type "Content-Type: application/json\nX-Custom: value"
    Then the literal "\n" characters are inserted
    And no actual newline is created
    And the text contains the backslash-n sequence

  Scenario: Insert text with quotes and escape sequences
    Given the request buffer is empty
    And I am in insert mode
    When I type "{\"name\": \"John\\nDoe\", \"email\": \"user@example.com\"}"
    Then all quotes and backslashes are properly inserted
    And the JSON structure is preserved as literal text
