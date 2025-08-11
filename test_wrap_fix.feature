Feature: Test wrap mode fix for double-byte characters

Scenario: Wrap mode should work with double-byte characters
    Given I have a running application
    And I enter insert mode with "i"
    And I type a long double-byte line: "あいうえおかきくけこあいうえおかきくけこあいうえおかきくけこあいうえおかきくけこあいうえおかきくけこ"
    And I exit insert mode with "Escape"
    And I enter command mode with ":"
    And I type "set wrap"
    And I press "Enter"
    Then I should be in Normal mode