Feature: Yank and Paste Operations
  Essential yank (copy) and paste functionality

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # Core yank/paste operations
  @skip
  Scenario: Yank and paste current line (yy, p)
    Given I am in Insert mode
    When I type "This line will be yanked"
    And I press Escape
    When I press "y" followed by "y"
    And I press "p"
    Then I should see two identical lines

  @skip
  Scenario: Yank in visual mode and paste
    Given I am in Insert mode
    When I type "Select this text"
    And I press Escape
    And I press "0"
    When I press "v"
    And I press "w"
    And I press "y"
    Then I should be in Normal mode
    When I press "$"
    And I press "p"
    Then the yanked text should be pasted after cursor

  @skip
  Scenario: Visual line mode yank (V, y, p)
    Given I am in Insert mode
    When I type "Line 1"
    And I press Enter
    And I type "Line 2"
    And I press Enter
    And I type "Line 3"
    And I press Escape
    When I press "k"
    And I press "V"
    And I press "y"
    And I press "G"
    And I press "p"
    Then "Line 2" should be pasted as a new line
# 
#   @skip
#   Scenario: Paste before cursor (P)
#     Given I am in Insert mode
#     When I type "Hello World"
#     And I press Escape
#     And I press "0"
#     And I press "w"
#     And I press "y" followed by "w"
#     When I press "0"
#     And I press "P"
#     Then I should see "World Hello World" in the request pane at line 1

  @skip
  Scenario: Character vs Line yank distinction
    Given I am in Insert mode
    When I type "Test line"
    And I press Escape
    # Character yank
    When I press "0"
    And I press "v"
    And I press "e"
    And I press "y"
    And I press "$"
    And I press "p"
    Then "Test" should be pasted inline
    # Line yank
    When I press "0"
    And I press "y" followed by "y"
    And I press "p"
    Then the entire line should be pasted as a new line

  # Integration with delete/cut operations (when dcut is enabled)
  Scenario: Cut operations populate yank buffer
    Given I am in Insert mode
    When I type "Delete this word"
    And I press Escape
    And I press "0"
    When I press "d" followed by "w"
    And I press "$"
    And I press "p"
    Then "Delete " should be pasted

  # Multi-byte support
  Scenario: Yank and paste with multi-byte characters
    Given I am in Insert mode
    When I type "こんにちは世界"
    And I press Escape
    And I press "0"
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "y"
    And I press "$"
    And I press "p"
    Then the Japanese text should be correctly pasted