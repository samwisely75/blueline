Feature: Normal Mode Commands
  Essential normal mode editing commands (x, D, dd, r, J, etc.)

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  # Character deletion with 'x'
  Scenario: Cut character at beginning of line
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    And I press "0"
    When I press "x"
    Then I should be in Normal mode
    And I should see "ello" in the request pane at line 1

  Scenario: Cut character at end of line
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hell" in the request pane at line 1

  Scenario: Cut character in middle of line
    Given I am in Insert mode
    When I type "ABCDE"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    When I press "x"
    Then I should be in Normal mode
    And I should see "ABDE" in the request pane at line 1

  Scenario: Cut multiple characters sequentially
    Given I am in Insert mode
    When I type "ABCDE"
    And I press Escape
    And I press "0"
    When I press "x"
    Then I should see "BCDE" in the request pane at line 1
    When I press "x"
    Then I should see "CDE" in the request pane at line 1
    When I press "x"
    Then I should see "DE" in the request pane at line 1

  Scenario: Cut character with multi-byte text
    Given I am in Insert mode
    When I type "こんにちは"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    When I press "x"
    Then I should see "こんちは" in the request pane at line 1

  # Delete to end of line with 'D'
  Scenario: Cut to end of line from middle
    Given I am in Insert mode
    When I type "Hello World"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    When I press "D"
    Then I should be in Normal mode
    And I should see "Hello " in the request pane at line 1

  Scenario: Cut to end of line from beginning
    Given I am in Insert mode
    When I type "Complete line"
    And I press Escape
    And I press "0"
    When I press "D"
    Then I should be in Normal mode
    And the request buffer should be empty

  Scenario: Cut from end of line
    Given I am in Insert mode
    When I type "Test"
    And I press Escape
    When I press "D"
    Then I should be in Normal mode
    And I should see "Tes" in the request pane at line 1

  # Delete entire line with 'dd'
  Scenario: Cut single line with dd command
    Given I am in Insert mode
    When I type "first line"
    And I press Enter
    And I type "second line"
    And I press Enter
    And I type "third line"
    And I press Escape
    And I press "k"
    When I press "d"
    And I press "d"
    Then I should see "first line" in the request pane at line 1
    And I should see "third line" in the request pane at line 2

  Scenario: Cut first line with dd command
    Given I am in Insert mode
    When I type "line one"
    And I press Enter
    And I type "line two"
    And I press Escape
    And I press "gg"
    When I press "d"
    And I press "d"
    Then I should see "line two" in the request pane at line 1

  Scenario: Cut last line with dd command
    Given I am in Insert mode
    When I type "line one"
    And I press Enter
    And I type "line two"
    And I press Escape
    When I press "d"
    And I press "d"
    Then I should see "line one" in the request pane at line 1

  # Replace character with 'r'
  Scenario: Replace first character
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    And I press "0"
    When I press "r"
    And I type "J"
    Then I should see "Jello" in the request pane at line 1

  Scenario: Replace last character
    Given I am in Insert mode
    When I type "World"
    And I press Escape
    When I press "r"
    And I type "s"
    Then I should see "Worls" in the request pane at line 1

  Scenario: Replace character in middle
    Given I am in Insert mode
    When I type "Test"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    When I press "r"
    And I type "x"
    Then I should see "Text" in the request pane at line 1

  # Join lines with 'J'
  Scenario: Join lines with J command
    Given I am in Insert mode
    When I type "Line 1"
    And I press Enter
    And I type "Line 2"
    And I press Escape
    And I press "k"
    When I press "J"
    Then I should see "Line 1 Line 2" in the request pane at line 1

  Scenario: Join multiple lines
    Given I am in Insert mode
    When I type "A"
    And I press Enter
    And I type "B"
    And I press Enter
    And I type "C"
    And I press Escape
    And I press "gg"
    When I press "J"
    Then I should see "A B" in the request pane at line 1
    When I press "J"
    Then I should see "A B C" in the request pane at line 1

  # Mode restrictions
  Scenario: Verify normal mode commands don't work in Insert mode
    Given I am in Insert mode
    When I type "Hello"
    And I type "x"
    Then I should see "Hellox" in the request pane at line 1
    And I should be in Insert mode

  # Edge cases
  Scenario: Commands on empty buffer
    Given the request buffer is empty
    When I press "x"
    Then I should be in Normal mode
    And the cursor should be at display line 1 display column 1
    When I press "D"
    Then I should be in Normal mode
    When I press "d"
    And I press "d"
    Then I should be in Normal mode

  # Paste operations after cut
  @skip
  Scenario: Cut and paste workflow
    Given I am in Insert mode
    When I type "Hello World"
    And I press Escape
    And I press "0"
    When I press "x"
    And I press "$"
    And I press "p"
    Then I should see "ello WorldH" in the request pane at line 1