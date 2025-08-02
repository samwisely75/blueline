Feature: Navigation Commands
  As a developer using blueline
  I want to navigate through HTTP request text using vim-style navigation commands
  So that I can efficiently position my cursor for editing and viewing

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  Scenario: Basic vim navigation
    Given the request buffer contains:
      """
      POST /api/users

      {"name": "John Doe"}
      """
    And I am in normal mode
    When I press "h"
    Then the cursor moves left
    When I press "l"
    Then the cursor moves right
    When I press "j"
    Then the cursor moves down
    When I press "k"
    Then the cursor moves up
    And I am still in normal mode

  Scenario: Line navigation
    Given the request buffer contains:
      """
      POST /api/users

      {"name": "John Doe"}
      """
    And I am in normal mode
    When I press "0"
    Then the cursor moves to the beginning of the line
    And I am still in normal mode
    When I press "$"
    Then the cursor moves to the end of the line
    And I am still in normal mode

  Scenario: Move to next word in request buffer
    Given the request buffer contains:
      """
      GET /api users
      """
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 4
    When I press "w"
    Then the cursor moves to column 8

  Scenario: Move to next word wraps to next line
    Given the request buffer contains:
      """
      GET /api
      users
      """
    And the cursor is at column 8
    When I press "w"
    Then the cursor moves to line 1 column 0

  Scenario: Move to next word in response buffer
    Given there is a response in the response pane from:
      """
      foo bar baz
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 4
    When I press "w"
    Then the cursor moves to column 8

  Scenario: Navigate response content
    Given I have executed a request that returned a large JSON response from:
      """
      GET /api/users
      """
    And I am in the response pane
    When I use vim navigation keys
    Then I can scroll through the response content
    And line numbers are visible

  Scenario: Scroll up with Ctrl+U
    Given the request buffer contains:
      """
      GET /api/users
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      """
    And I am in normal mode
    And the cursor is at line 15
    When I press "Ctrl+U"
    Then the cursor moves up by half a page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Ctrl+D
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Ctrl+D"
    Then the cursor moves down by half a page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Ctrl+F
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Ctrl+F"
    Then the cursor moves down by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll up with Ctrl+B
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 25
    When I press "Ctrl+B"
    Then the cursor moves up by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll down with Page Down key
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "Page Down"
    Then the cursor moves down by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Scroll up with Page Up key
    Given the request buffer contains:
      """
      {"line": 1}
      {"line": 2}
      {"line": 3}
      {"line": 4}
      {"line": 5}
      {"line": 6}
      {"line": 7}
      {"line": 8}
      {"line": 9}
      {"line": 10}
      {"line": 11}
      {"line": 12}
      {"line": 13}
      {"line": 14}
      {"line": 15}
      {"line": 16}
      {"line": 17}
      {"line": 18}
      {"line": 19}
      {"line": 20}
      {"line": 21}
      {"line": 22}
      {"line": 23}
      {"line": 24}
      {"line": 25}
      {"line": 26}
      {"line": 27}
      {"line": 28}
      {"line": 29}
      {"line": 30}
      """
    And I am in normal mode
    And the cursor is at line 25
    When I press "Page Up"
    Then the cursor moves up by a full page
    And the scroll offset is adjusted accordingly
    And I am still in normal mode

  Scenario: Go to top with gg command
    Given the request buffer contains:
      """
      GET /api/users HTTP/1.1
      {"query": "search term"}
      """
    And I am in normal mode
    And the cursor is at line 5
    When I press "g"
    And I press "g"
    Then the cursor moves to the first line
    And the cursor is at column 0
    And the scroll offset is reset to 0
    And I am still in normal mode

  Scenario: Go to bottom with G command
    Given the request buffer contains:
      """
      GET /api/users HTTP/1.1
      {"query": "search term"}
      Last line of content
      """
    And I am in normal mode
    And the cursor is at line 0
    When I press "G"
    Then the cursor moves to the last line
    And the cursor is at column 0
    And I am still in normal mode

  Scenario: Navigate right through Japanese characters in request pane
    Given the request buffer contains "きんようび"
    And cursor is in front of `き`
    When I press "l"
    Then the cursor moves in front of `ん`
    When I press "l"
    Then the cursor moves in front of `よ`
    When I press "l"
    Then the cursor moves in front of `う`
    When I press "l"
    Then the cursor moves in front of `び`

  Scenario: Navigate left through Japanese characters in response pane
    Given there is a response in the response pane from "きんようび"
    And I am in the response pane
    And cursor is in front of `び`
    When I press "h"
    Then the cursor moves in front of `う`
    When I press "h"
    Then the cursor moves in front of `よ`
    When I press "h"
    Then the cursor moves in front of `ん`
    When I press "h"
    Then the cursor moves in front of `き`

  Scenario: Word movement with Japanese text in request pane
    Given the request buffer contains "こんにちは。私、名前 Borat です"
    And cursor is in front of `こ`
    When I press "w"
    Then the cursor moves in front of `私` by skipping the series of regular characters and termination char `。`
    When I press "w"
    Then the cursor moves in front of `名前` by skipping Japanese punctuation character `、`
    When I press "w"
    Then the cursor moves in front of `Borat`
    When I press "w"
    Then the cursor moves in front of `です`

  Scenario: Backward word movement with Japanese text in request pane
    Given the request buffer contains "こんにちは。私、名前 Borat です"
    And cursor is in front of `Borat`
    When I press "b"
    Then the cursor moves in front of `名前` by skipping Japanese punctuation character `、`
    When I press "b"
    Then the cursor moves in front of `私`
    When I press "b"
    Then the cursor moves in front of `こ`

  Scenario: End of word movement with Japanese text
    Given the request buffer contains "こんにちは。私、名前 Borat です"
    And cursor is in front of `こ`
    When I press "e"
    Then the cursor moves to end of `こんにちは`
    When I press "e"
    Then the cursor moves to end of `私`
    When I press "e"
    Then the cursor moves to end of `名前`
    When I press "e"
    Then the cursor moves to end of `Borat`
    When I press "e"
    Then the cursor moves to end of `です`
