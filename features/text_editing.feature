Feature: Text Editing Operations
  As a developer using blueline
  I want to edit HTTP request text efficiently with vim-like commands
  So that I can compose and modify requests quickly

  Background:
    Given blueline is running with default profile
    And I am in the request pane

  # === INSERT MODE SCENARIOS ===
  
  Scenario: Enter insert mode and type basic text
    Given I am in normal mode
    When I press "i"
    And I type "GET /api/users"
    And I press Escape
    Then the screen should not be blank
    And I should see "GET /api/users" in the request pane
    And the cursor should be positioned correctly

  Scenario: Insert special characters that might cause issues
    Given I am in insert mode
    And the request buffer is empty
    When I type "GET /api/users?email=user@example.com"
    Then the text appears in the request buffer
    And the at-sign "@" is properly inserted

  Scenario: Insert backtick characters
    Given I am in insert mode
    And the request buffer is empty
    When I type "Authorization: Bearer `token123`"
    Then the text appears in the request buffer
    And the backticks "`" are properly inserted

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
    When I press Enter
    Then a new line is created
    And the cursor moves to the beginning of the new line
    And the previous line remains unchanged

  Scenario: Multi-line editing with Enter
    Given I am in insert mode
    When I type "GET /api/test HTTP/1.1"
    And I press Enter to create a new line
    And I type "Host: example.com"
    Then the screen should not be blank
    And I should see both lines correctly formatted
    And line numbers should be displayed

  Scenario: Insert multiline request with newlines
    Given I am in insert mode
    And the request buffer is empty
    When I type:
      """
      POST /api/users
      Content-Type: application/json

      {"name": "John"}
      """
    Then the request buffer contains multiple lines
    And empty lines are preserved
    And JSON formatting is maintained

  Scenario: Insert text with quotes and escape sequences
    Given I am in insert mode
    And the request buffer is empty
    When I type "{\"name\": \"John\\nDoe\", \"email\": \"user@example.com\"}"
    Then all quotes and backslashes are properly inserted
    And the JSON structure is preserved as literal text

  Scenario: Handle problematic characters that might cause parsing errors
    Given I am in insert mode
    And the request buffer is empty
    When I type "Content-Type: application/json\nX-Custom: value"
    Then the literal "\n" characters are inserted
    And no actual newline is created
    And the text contains the backslash-n sequence

  # === DELETION SCENARIOS ===

  Scenario: Delete character with Backspace at end of line
    Given I am in insert mode
    And the request buffer contains "GET /api/userss"
    And the cursor is at the end of the line
    When I press Backspace
    Then the last "s" character is deleted
    And the cursor moves back one position
    And the text becomes "GET /api/users"

  Scenario: Delete character with Backspace in middle of line
    Given I am in insert mode
    And the request buffer contains "GET /appi/users"
    And the cursor is positioned after the extra "i"
    When I press Backspace
    Then the extra "i" is deleted
    And the text becomes "GET /api/users"

  Scenario: Text deletion with backspace multiple times
    Given I am in insert mode
    And I have typed "Hello World"
    When I press backspace 5 times
    Then the screen should not be blank
    And I should see "Hello " in the request pane
    And the cursor should be after the space

  Scenario: Join lines with Backspace at line start
    Given I am in insert mode
    And the request buffer contains:
      """
      GET /api/users
      Second line
      """
    And the cursor is at the beginning of the second line
    When I press Backspace
    Then the lines are joined together
    And the text becomes "GET /api/usersSecond line"
    And the cursor position is correct

  Scenario: Backspace at beginning of first line should not delete
    Given I am in insert mode
    And the request buffer contains "GET /api/users"
    And the cursor is at the beginning of the first line
    When I press Backspace
    Then no character is deleted
    And the text remains "GET /api/users"
    And the cursor stays at the beginning

  Scenario: Backspace on blank line deletes entire line
    Given I am in insert mode
    And the request buffer contains:
      """
      GET /api/users

      {"name": "John"}
      """
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
    Given I am in insert mode
    And the request buffer contains:
      """
      GET /api/users


      {"name": "John"}
      """
    And the cursor is on the second blank line (line 3)
    When I press Backspace
    Then only the current blank line is deleted
    And the cursor moves to the end of the previous line (first blank line)
    And the text becomes:
      """
      GET /api/users

      {"name": "John"}
      """

  Scenario: Text deletion with delete key
    Given I have text "Hello World" in the request pane
    And the cursor is at the beginning
    When I press the delete key 6 times
    Then the screen should not be blank
    And I should see "World" in the request pane

  # === NAVIGATION SCENARIOS ===

  Scenario: Line navigation with j/k keys
    Given I am in normal mode
    And I have multiple lines of text:
      """
      Line 1
      Line 2  
      Line 3
      Line 4
      """
    When I press "j" to move down
    Then the cursor should be on line 2
    And the screen should not be blank
    When I press "k" to move up
    Then the cursor should be on line 1
    And the screen should not be blank

  Scenario: Character navigation with h/l keys
    Given I am in normal mode
    And I have text "Hello World" on one line
    And the cursor is at the beginning
    When I press "l" 6 times
    Then the cursor should be after "Hello "
    And the screen should not be blank
    When I press "h" 3 times
    Then the cursor should be after "Hel"
    And the screen should not be blank

  Scenario: Word-based navigation
    Given I am in normal mode
    And I have text "The quick brown fox jumps"
    And the cursor is at the beginning
    When I press "w" to move to next word
    Then the cursor should be at "quick"
    And the screen should not be blank
    When I press "b" to move to previous word  
    Then the cursor should be at "The"
    And the screen should not be blank

  Scenario: Line beginning and end navigation
    Given I am in normal mode
    And I have text "Hello World" on one line
    And the cursor is in the middle
    When I press "0" to go to line beginning
    Then the cursor should be at the start of the line
    And the screen should not be blank
    When I press "$" to go to line end
    Then the cursor should be at the end of the line
    And the screen should not be blank

  # === ADVANCED FEATURES ===

  Scenario: Undo functionality (if implemented)
    Given I have typed some text
    When I delete part of the text
    And I press "u" for undo
    Then the deleted text should be restored
    And the screen should not be blank

  Scenario: Copy and paste operations (if implemented)
    Given I have text "Hello World"
    When I select the text in visual mode
    And I copy it with "y"
    And I move to a new position
    And I paste with "p"
    Then the text should be duplicated
    And the screen should not be blank