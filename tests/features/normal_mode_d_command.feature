Feature: Normal mode 'D' command (cut to end of line)

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  Scenario: Cut from middle of line to end
    Given I am in Insert mode
    When I type "Hello World"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 6
    When I press "D"
    Then I should be in Normal mode
    And I should see "Hello " in the request pane at line 1
    And the cursor should be at display line 1 display column 6
    When I press "p"
    Then I should see "Hello World" in the request pane at line 1

  Scenario: Cut from beginning of line to end (whole line)
    Given I am in Insert mode
    When I type "Complete line"
    And I press Escape
    And I press "0"
    When I press "D"
    Then I should be in Normal mode
    And I should see "" in the request pane at line 1
    And the cursor should be at display line 1 display column 1
    When I press "p"
    Then I should see "Complete line" in the request pane at line 1

  Scenario: Cut from end of line (no-op)
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    Then the cursor should be at display line 1 display column 5
    When I press "D"
    Then I should be in Normal mode
    And I should see "Hello" in the request pane at line 1
    And the cursor should be at display line 1 display column 5

  Scenario: Cut with multi-byte characters
    Given I am in Insert mode
    When I type "こんにちは World"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 6
    When I press "D"
    Then I should be in Normal mode
    And I should see "こんにちは" in the request pane at line 1
    And the cursor should be at display line 1 display column 6
    When I press "p"
    Then I should see "こんにちは World" in the request pane at line 1

  Scenario: Cut from empty line (no-op)
    Given the request buffer is empty
    When I press "D"
    Then I should be in Normal mode
    And I should see "" in the request pane at line 1
    And the cursor should be at display line 1 display column 1

  Scenario: Cut multiple times sequentially
    Given I am in Insert mode
    When I type "ABCDEFGH"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    When I press "D"
    Then I should see "ABC" in the request pane at line 1
    And the cursor should be at display line 1 display column 4
    When I press "D"
    Then I should see "ABC" in the request pane at line 1
    And the cursor should be at display line 1 display column 3

  Scenario: Verify 'D' only works in Normal mode
    Given I am in Insert mode
    When I type "Hello"
    And I type "D"
    Then I should see "HelloD" in the request pane at line 1
    And I should be in Insert mode

  Scenario: Verify 'D' only works in Request pane
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    And I press Tab
    Then I should be in the Response pane
    When I press "D"
    Then I should be in Normal mode
    And I should see "Hello" in the request pane at line 1

  Scenario: Cut from line with trailing spaces
    Given I am in Insert mode
    When I type "Hello   "
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    When I press "D"
    Then I should see "Hel" in the request pane at line 1
    When I press "p"
    Then I should see "Hello   " in the request pane at line 1

  Scenario: Cut and paste preserves yank buffer
    Given I am in Insert mode
    When I type "First line"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    When I press "D"
    Then I should see "First " in the request pane at line 1
    When I press "o"
    And I type "Second line"
    And I press Escape
    When I press "p"
    Then I should see "line" in the request pane at line 2