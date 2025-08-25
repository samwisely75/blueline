Feature: Cursor and Navigation Commands
  Comprehensive cursor movement and navigation testing

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # Basic Navigation
  Scenario: Basic vim navigation (h,j,k,l)
    Given I am in Insert mode
    When I type "Line 1"
    And I press Enter
    And I type "Line 2"
    And I press Enter
    And I type "Line 3"
    And I press Escape
    And I press "k"
    Then the cursor should move up one line
    When I press "j"
    Then the cursor should move down one line
    When I press "h"
    Then the cursor should move left one character
    When I press "l"
    Then the cursor should move right one character

  # Arrow Keys
  Scenario: Arrow keys work in all modes
    Given I am in Insert mode
    When I type "Test"
    And I press the Left arrow key
    Then the cursor should move left one character
    When I press Escape
    And I am in Normal mode
    And I press the Right arrow key
    Then the cursor should move right one character

  # Line Navigation
  Scenario Outline: Line navigation commands
    Given I am in Insert mode
    When I type "Start Middle End"
    And I press Escape
    And I press "<command>"
    Then the cursor should be at <position>

    Examples:
      | command | position                |
      | 0       | the beginning of line   |
      | $       | the end of line         |
      | Home    | the beginning of line   |
      | End     | the end of line         |

  # Word Navigation
  Scenario: Word navigation (w, b, e)
    Given I am in Insert mode
    When I type "one two three four"
    And I press Escape
    And I press "0"
    When I press "w"
    Then the cursor should be at the start of "two"
    When I press "e"
    Then the cursor should be at the end of "two"
    When I press "b"
    Then the cursor should be at the start of "two"

  # Document Navigation
  Scenario: Document navigation (gg, G)
    Given I am in Insert mode
    When I type "First line"
    And I press Enter
    And I type "Middle line"
    And I press Enter
    And I type "Last line"
    And I press Escape
    When I press "g" followed by "g"
    Then the cursor should be at the first line
    When I press "G"
    Then the cursor should be at the last line

  # Multi-byte Character Navigation
  Scenario: Navigation with Japanese characters
    Given I am in Insert mode
    When I type "こんにちは世界"
    And I press Escape
    And I press "0"
    When I press "l"
    Then the cursor should be at display line 1 display column 3
    When I press "l"
    Then the cursor should be at display line 1 display column 5
    When I press "$"
    Then the cursor should be at display line 1 display column 13

  # Mixed ASCII and Multi-byte
  Scenario: Navigation with mixed characters
    Given I am in Insert mode
    When I type "Hello世界World"
    And I press Escape
    When I press "0"
    And I press "w"
    Then the cursor should be on the first multi-byte character
    When I press "e"
    Then the cursor should be at the end of the multi-byte word

  # Page Navigation
  Scenario: Page up and down (Ctrl-F, Ctrl-B)
    Given I have multiple pages of content
    When I press "Ctrl+f"
    Then the view should scroll down one page
    When I press "Ctrl+b"
    Then the view should scroll up one page

  # Cursor Position Memory
  Scenario: Cursor maintains virtual column on vertical movement
    Given I am in Insert mode
    When I type "Short"
    And I press Enter
    And I type "This is a very long line"
    And I press Enter
    And I type "Short"
    And I press Escape
    And I press "k"
    And I press "$"
    When I press "j"
    Then the cursor should be at the end of the shorter line
    When I press "k"
    Then the cursor should return to the end of the long line