Feature: Mode Transitions
    As a user of Blueline REPL
    I want to switch between different modes
    So that I can interact with the application appropriately

    Background:
        Given the application is started with default settings

    Scenario: Initial mode is Insert
        When the application starts
        Then I should be in Insert mode
        And the request pane should show line number "1" in column 3
        And the request pane should show "~" for empty lines
        And there should be a blinking block cursor at column 4
        And the status bar should show "REQUEST | 1:1" aligned to the right
        And there should be no response pane visible

    Scenario: Switch from Insert to Command mode
        Given I am in Insert mode
        When I press Escape
        Then I should be in Command mode
        And the cursor should change appearance

    Scenario: Switch from Command back to Insert mode
        Given I am in Command mode
        When I press "i"
        Then I should be in Insert mode
        And the cursor should change appearance

    Scenario: Execute command in Insert mode
        Given I am in Insert mode
        When I type "echo hello"
        And I press Enter
        Then I should see "hello" in the output
        And I should remain in Insert mode