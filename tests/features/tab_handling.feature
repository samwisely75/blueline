Feature: Tab handling and expandtab configuration
  As a user
  I want to configure tab behavior
  So that I can control whether tabs are inserted as spaces or tab characters

  Background:
    Given the application is running
    And I am in the Request pane
    And I am in Normal mode

  Scenario: Setting tabstop width
    When I press ":"
    Then I should be in Command mode
    When I type "set tabstop 2"
    And I press "Enter"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the cursor should be at column 2
    When I press "Tab"
    Then the cursor should be at column 4

  Scenario: Enable expandtab to insert spaces instead of tabs
    When I press ":"
    Then I should be in Command mode
    When I type "set expandtab on"
    And I press "Enter"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the buffer should contain 4 spaces at the cursor position
    And the cursor should be at column 4

  Scenario: Disable expandtab to insert tab characters
    When I press ":"
    Then I should be in Command mode
    When I type "set expandtab off"
    And I press "Enter"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the buffer should contain a tab character at the cursor position
    And the cursor should be at column 4

  Scenario: Expandtab uses current tabstop width
    When I press ":"
    Then I should be in Command mode
    When I type "set tabstop 2"
    And I press "Enter"
    Then I should be in Normal mode
    When I press ":"
    Then I should be in Command mode
    When I type "set expandtab on"
    And I press "Enter"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the buffer should contain 2 spaces at the cursor position
    And the cursor should be at column 2

  Scenario: Convert existing tabs to spaces when expandtab is enabled
    Given I have text "hello\tworld\tthere" in the buffer
    When I press ":"
    Then I should be in Command mode
    When I type "set expandtab on"
    And I press "Enter"
    Then the tabs in the buffer should be replaced by spaces
    And the buffer should contain "hello    world    there"

  Scenario: Expandtab state persists across mode changes
    When I press ":"
    Then I should be in Command mode
    When I type "set expandtab on"
    And I press "Enter"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the buffer should contain 4 spaces at the cursor position
    When I press "Escape"
    Then I should be in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I press "Tab"
    Then the buffer should contain 4 spaces at the cursor position