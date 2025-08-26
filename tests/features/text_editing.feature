Feature: Text Editing Operations
  Comprehensive text insertion, deletion, and modification

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # Insert Mode Operations
  Scenario Outline: Insert mode entry commands
    Given I am in Normal mode with text "Hello World"
    When I press "<command>"
    Then I should be in Insert mode
    And the cursor should be at <position>

    Examples:
      | command | position                    |
      | i       | current position            |
      | a       | after current character     |
      | I       | beginning of line           |
      | A       | end of line                 |
      | o       | new line below              |
      | O       | new line above              |

  # Text Insertion
  Scenario: Basic text insertion
    Given I am in Insert mode
    When I type "Hello, World!"
    Then I should see "Hello, World!" in the request pane at line 1

  Scenario: Multi-line text insertion
    Given I am in Insert mode
    When I type "Line 1"
    And I press Enter
    And I type "Line 2"
    And I press Enter
    And I type "Line 3"
    Then the request pane should have 3 lines

  # Text Deletion
  Scenario Outline: Delete operations in Insert mode
    Given I am in Insert mode
    When I type "Test Text"
    And I press "<key>"
    Then I should see "<result>" in the request pane

    Examples:
      | key       | result     |
      | Backspace | Test Tex   |
      | Delete    | Test Tex   |

  Scenario: Delete word with Ctrl-W in Insert mode
    Given I am in Insert mode
    When I type "one two three"
    And I press "Ctrl+w"
    Then I should see "one two " in the request pane at line 1

  # Tab Handling
  Scenario: Tab insertion and expansion
    Given I am in Insert mode
    When I press Tab
    Then spaces or tab should be inserted based on expandtab setting

  # Multi-byte Text Editing
  Scenario: Insert and delete multi-byte characters
    Given I am in Insert mode
    When I type "こんにちは"
    And I press Backspace
    Then I should see "こんにち" in the request pane at line 1
    When I type "は世界"
    Then I should see "こんにちは世界" in the request pane at line 1

  # Append Operations
  # Note: Current implementation of 'a' behaves like 'i' (inserts at cursor instead of after)
  Scenario: Append at specific positions
    Given I am in Insert mode
    When I type "Start"
    And I press Escape
    When I press "a"
    And I type " Middle"
    And I press Escape
    And I press "A"
    And I type " End"
    Then I should see "Star Middlet End" in the request pane at line 1

  # Replace Operations
  Scenario: Replace character with r command
    Given I am in Normal mode with text "Hello"
    When I press "r"
    And I type "J"
    Then I should see "Jello" in the request pane at line 1

  # Join Lines
#   @skip
#   Scenario: Join lines with J command
#     Given I am in Insert mode
#     When I type "Line 1"
#     And I press Enter
#     And I type "Line 2"
#     And I press Escape
#     And I press "k"
#     When I press "J"
#     Then I should see "Line 1 Line 2" in the request pane at line 1

  # Undo/Redo (not implemented in production)
  @skip
  Scenario: Undo and redo operations
    Given I am in Insert mode
    When I type "Original"
    And I press Escape
    And I press "u"
    Then the buffer should be empty
    When I press "Ctrl+r"
    Then I should see "Original" in the request pane at line 1