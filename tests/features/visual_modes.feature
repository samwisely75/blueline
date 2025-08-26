Feature: Visual Mode Operations
  Comprehensive visual, visual line, and visual block mode testing

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # Visual Character Mode
  Scenario: Enter and exit visual character mode
    Given the request buffer contains:
      """
      Sample text for testing
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    Then I should be in Visual mode
    When I press "Escape"
    Then I should be in Normal mode

  Scenario Outline: Visual character selection and deletion
    Given the request buffer contains:
      """
      <text>
      """
    And the cursor is at display line 1 display column <start>
    When I press "v"
    And I press "<motion>"
    And I press "d"
    Then I should be in Normal mode
    And I should see "<result>" in the request pane at line 1

    Examples:
      | text        | start | motion | result   |
      | HELLO       | 2     |        | HLLO     |
      | TEST 1234   | 1     | lllll  | 234      |
      | Hello World | 1     | llll   |  World   |

  # Visual Line Mode
  Scenario: Enter and exit visual line mode
    Given the request buffer contains:
      """
      Line 1
      Line 2
      Line 3
      """
    And the cursor is at display line 2 display column 1
    When I press "V"
    Then I should be in Visual Line mode
    When I press "Escape"
    Then I should be in Normal mode

#   @skip  # Buffer initialization issue
#   Scenario: Delete single line with Visual Line mode
#     Given the request buffer contains:
#       """
#       Line 1
#       Line 2 to delete
#       Line 3
#       """
#     And the cursor is at display line 2 display column 1
#     When I press "V"
#     And I press "d"
#     Then I should be in Normal mode
#     And I should see "Line 1" in the request pane at line 1
#     And I should see "Line 3" in the request pane at line 2
# 
#   @skip  # Buffer initialization issue
#   Scenario: Delete multiple lines with Visual Line mode
#     Given the request buffer contains:
#       """
#       Keep this line
#       Delete line 1
#       Delete line 2
#       Delete line 3
#       Keep this line too
#       """
#     And the cursor is at display line 2 display column 1
#     When I press "V"
#     And I press "j"
#     And I press "j"
#     And I press "d"
#     Then I should be in Normal mode
#    And I should see "Keep this line" in the request pane at line 1
#    And I should see "Keep this line too" in the request pane at line 2

  # Visual Block Mode (currently skipped in tests)
  @skip
  Scenario: Enter and exit visual block mode
    Given the request buffer contains:
      """
      Column 1
      Column 2
      Column 3
      """
    And the cursor is at display line 1 display column 1
    When I press "Ctrl-v"
    Then I should be in Visual Block mode
    When I press "Escape"
    Then I should be in Normal mode

  # Multi-byte character support
  Scenario: Visual selection with multi-byte characters
    Given the request buffer contains:
      """
      こんにちは World
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "d"
    Then I should be in Normal mode
    And I should see "にちは World" in the request pane at line 1

  # Integration with other operations
  Scenario: Visual selection and yank
    Given the request buffer contains:
      """
      Copy this text
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "y"
    Then I should be in Normal mode
    And the status message should contain "4 characters yanked"

#   @skip  # Buffer initialization issue
#   Scenario: Visual line and cut
#     Given the request buffer contains:
#       """
#       Line 1
#       Line 2 to cut
#       Line 3
#       """
#     And the cursor is at display line 2 display column 1
#     When I press "V"
#     And I press "x"
#     Then I should be in Normal mode
#     And I should see "Line 1" in the request pane at line 1
#     And I should see "Line 3" in the request pane at line 2