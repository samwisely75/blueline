Feature: Debug wrap mode functionality

Scenario: Test wrap mode with debug logging
    Given I have a running application
    And I enter insert mode with "i"
    And I type a long line: "This is a very long line that should definitely wrap when wrap mode is enabled. こんにちは世界"
    And I exit insert mode with "Escape"
    And I enter command mode with ":"
    And I type "set wrap"
    And I press "Enter"
    Then I should be in Normal mode