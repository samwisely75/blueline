Feature: Text Editing Operations
  As a user of blueline
  I want to edit text efficiently with vim-like commands
  So that I can compose HTTP requests quickly

  Background:
    Given blueline is launched in a terminal
    And I am in the request pane in normal mode

  Scenario: Basic text insertion
    When I press "i" to enter insert mode
    And I type "Hello World"
    And I press Escape to exit insert mode
    Then the screen should not be blank
    And I should see "Hello World" in the request pane
    And the cursor should be positioned correctly

  Scenario: Text deletion with backspace
    Given I am in insert mode
    And I have typed "Hello World"
    When I press backspace 5 times
    Then the screen should not be blank
    And I should see "Hello " in the request pane
    And the cursor should be after the space

  Scenario: Text deletion with delete key
    Given I have text "Hello World" in the request pane
    And the cursor is at the beginning
    When I press the delete key 6 times
    Then the screen should not be blank
    And I should see "World" in the request pane

  Scenario: Line navigation with j/k keys
    Given I have multiple lines of text:
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
    Given I have text "Hello World" on one line
    And the cursor is at the beginning
    When I press "l" 6 times
    Then the cursor should be after "Hello "
    And the screen should not be blank
    When I press "h" 3 times
    Then the cursor should be after "Hel"
    And the screen should not be blank

  Scenario: Word-based navigation
    Given I have text "The quick brown fox jumps"
    And the cursor is at the beginning
    When I press "w" to move to next word
    Then the cursor should be at "quick"
    And the screen should not be blank
    When I press "b" to move to previous word  
    Then the cursor should be at "The"
    And the screen should not be blank

  Scenario: Line beginning and end navigation
    Given I have text "Hello World" on one line
    And the cursor is in the middle
    When I press "0" to go to line beginning
    Then the cursor should be at the start of the line
    And the screen should not be blank
    When I press "$" to go to line end
    Then the cursor should be at the end of the line
    And the screen should not be blank

  Scenario: Multi-line editing with Enter
    Given I am in insert mode
    When I type "GET /api/test HTTP/1.1"
    And I press Enter to create a new line
    And I type "Host: example.com"
    Then the screen should not be blank
    And I should see both lines correctly formatted
    And line numbers should be displayed

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