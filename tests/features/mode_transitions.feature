Feature: Mode Transitions
    As a user of Blueline REPL
    I want to switch between different modes
    So that I can interact with the application appropriately

    Background:
        Given the application is started with default settings
        And the request buffer is empty

    Scenario: Switch from Normal to Insert and back to Normal
        Given I am in Normal mode
        When I press "i"
        Then I should be in Insert mode
        And the cursor should change appearance
        When I type "GET /api/users"
        Then I should see "GET /api/users" in the output
        When I press Escape
        Then I should be in Normal mode
        And the cursor should change appearance

    Scenario: Switch from Normal to Visual mode and back to Normal
        Given I am in Normal mode
        When I press "i"
        Then I should be in Insert mode
        When I type "hello world"
        And I press Escape
        Then I should be in Normal mode
        When I press "v"
        Then I should be in Visual mode
        And the cursor should change appearance
        When I press "$"
        Then the selection should expand
        And I should see "hello world" highlighted
        When I press Escape
        Then I should be in Normal mode
        And the cursor should change appearance

    Scenario: Switch from Normal to Command mode
        Given I am in Normal mode
        When I press ":"
        Then I should be in Command mode
        And I should see ":" in the status line
        When I type "help"
        Then I should see ":help" in the status line
        When I press Enter
        Then I should see the help message in the output
        And I should be in Normal mode