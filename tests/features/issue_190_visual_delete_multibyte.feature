Feature: Issue #190 - Visual mode delete with multi-byte characters

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  Scenario: Delete double-byte characters at end of line in Visual mode (Issue #190)
    Given I am in Insert mode
    When I type "あいうえおかきくけこ"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l" 
    And I press "l"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 6
    When I press "v"
    Then I should be in Visual mode
    When I press "l"
    And I press "l" 
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 10
    When I press "d"
    Then I should be in Normal mode
    And I should see "あいうえお" in the request pane at line 1
    And the cursor should be at display line 1 display column 6

  Scenario: Delete mixed multi-byte and ASCII characters in Visual mode
    Given I am in Insert mode  
    When I type "abc漢字defかきく"
    And I press Escape
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 4
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    Then the cursor should be at display line 1 display column 9
    When I press "d"
    Then I should be in Normal mode
    And I should see "abcかきく" in the request pane at line 1
    And the cursor should be at display line 1 display column 4