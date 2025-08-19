Feature: Normal mode 'x' command (cut character)

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  Scenario: Cut single character in middle of line
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
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hello orld" in the request pane at line 1
    And the cursor should be at display line 1 display column 6
    When I press "p"
    Then I should see "Hello Woorld" in the request pane at line 1

  Scenario: Cut character at beginning of line
    Given I am in Insert mode  
    When I type "Hello"
    And I press Escape
    And I press "0"
    When I press "x"
    Then I should be in Normal mode
    And I should see "ello" in the request pane at line 1
    And the cursor should be at display line 1 display column 1
    When I press "p"
    Then I should see "eHllo" in the request pane at line 1

  Scenario: Cut character at end of line
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    Then the cursor should be at display line 1 display column 5
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hell" in the request pane at line 1
    And the cursor should be at display line 1 display column 4
    When I press "p"
    Then I should see "Helo" in the request pane at line 1

  Scenario: Cut character with multi-byte characters
    Given I am in Insert mode
    When I type "こんにちは"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 3
    When I press "x" 
    Then I should be in Normal mode
    And I should see "こんちは" in the request pane at line 1
    And the cursor should be at display line 1 display column 3
    When I press "p"
    Then I should see "こんにちは" in the request pane at line 1

  Scenario: Cut multi-byte character at end of line adjusts cursor properly
    Given I am in Insert mode
    When I type "abc漢字"
    And I press Escape
    Then the cursor should be at display line 1 display column 5
    When I press "x"
    Then I should see "abc漢" in the request pane at line 1
    And the cursor should be at display line 1 display column 4
    When I press "x"
    Then I should see "abc" in the request pane at line 1
    And the cursor should be at display line 1 display column 3

  Scenario: Try to cut character at end of empty line (no-op)
    Given the request buffer is empty
    When I press "x"
    Then I should be in Normal mode
    And I should see "" in the request pane at line 1
    And the cursor should be at display line 1 display column 1

  Scenario: Try to cut character beyond end of line (no-op)
    Given I am in Insert mode
    When I type "Hi"
    And I press Escape
    And I press "$"
    And I press "l"
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hi" in the request pane at line 1

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

  Scenario: Cut from end of line adjusts cursor position
    Given I am in Insert mode
    When I type "123456"
    And I press Escape
    Then the cursor should be at display line 1 display column 6
    When I press "x"
    Then I should see "12345" in the request pane at line 1
    And the cursor should be at display line 1 display column 5
    When I press "x"
    Then I should see "1234" in the request pane at line 1
    And the cursor should be at display line 1 display column 4
    When I press "x"
    Then I should see "123" in the request pane at line 1
    And the cursor should be at display line 1 display column 3

  Scenario: Verify 'x' only works in Normal mode
    Given I am in Insert mode
    When I type "Hello"
    And I type "x"
    Then I should see "Hellox" in the request pane at line 1
    And I should be in Insert mode

  Scenario: Verify 'x' only works in Request pane  
    Given I am in Insert mode
    When I type "Hello"
    And I press Escape
    And I press Tab
    Then I should be in the Response pane
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hello" in the request pane at line 1