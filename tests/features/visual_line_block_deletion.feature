Feature: Visual Line and Block deletion operations

  Scenario: Delete single line with Visual Line mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      Line 1
      Line 2 to delete
      Line 3
      """
    And the cursor is at display line 2 display column 1
    When I press "V"
    Then I should be in Visual Line mode
    When I press "d"
    Then I should be in Normal mode
    And I should see "Line 1" in the request pane at line 1
    And I should see "Line 3" in the request pane at line 2

  Scenario: Delete multiple lines with Visual Line mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      Keep this line
      Delete line 1
      Delete line 2
      Delete line 3
      Keep this line too
      """
    And the cursor is at display line 2 display column 1
    When I press "V"
    Then I should be in Visual Line mode
    When I press "j"
    When I press "j"
    When I press "d"
    Then I should be in Normal mode
    And I should see "Keep this line" in the request pane at line 1
    And I should see "Keep this line too" in the request pane at line 2

  Scenario: Cut lines with Visual Line mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      Line 1
      Line 2 to cut
      Line 3 to cut
      Line 4
      """
    And the cursor is at display line 2 display column 1
    When I press "V"
    When I press "j"
    When I press "x"
    Then I should be in Normal mode
    And I should see "Line 1" in the request pane at line 1
    And I should see "Line 4" in the request pane at line 2

  Scenario: Delete rectangular block with Visual Block mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      123456789
      abcdefghi
      ABCDEFGHI
      """
    And the cursor is at display line 1 display column 3
    When I press "Ctrl-v"
    Then I should be in Visual Block mode
    When I press "l"
    When I press "l"
    When I press "j"
    When I press "j"
    When I press "d"
    Then I should be in Normal mode
    And I should see "123789" in the request pane at line 1
    And I should see "abcghi" in the request pane at line 2
    And I should see "ABCGHI" in the request pane at line 3

  Scenario: Cut rectangular block with Visual Block mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      Hello World
      Test  Block
      More  Lines
      """
    And the cursor is at display line 1 display column 6
    When I press "Ctrl-v"
    When I press "l"
    When I press "l"
    When I press "l"
    When I press "j"
    When I press "j"
    When I press "x"
    Then I should be in Normal mode
    And I should see "Hello rld" in the request pane at line 1
    And I should see "Test  ock" in the request pane at line 2
    And I should see "More  nes" in the request pane at line 3