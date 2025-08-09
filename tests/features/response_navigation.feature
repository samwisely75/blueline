Feature: Response Pane Navigation
  As a developer using blueline
  I want to navigate within the response pane using vim-style commands
  So that I can efficiently examine HTTP response content

  Background:
    Given the application is started with default settings
    And I am in Normal mode
    And wrap is off

  Scenario: Word navigation in response pane works correctly (ASCII text, wrap off)
    Given there is a response in the response pane from:
      """
      Hello world this is a test
      """
    And I am in the response pane
    And the cursor is at display line 0 display column 0
    When I press "w"
    Then the cursor moves to display line 0 display column 6
    When I press "w" 
    Then the cursor moves to display line 0 display column 12
    When I press "b"
    Then the cursor moves to display line 0 display column 6
    When I press "b"
    Then the cursor moves to display line 0 display column 0

  Scenario: Vertical navigation clamps to line length in response pane (ASCII text, wrap off)
    Given there is a response in the response pane from:
      """
      This is a very long line with many words
      Short
      Another long line here
      """
    And I am in the response pane
    And the cursor is at display line 0 display column 35
    When I press "j"
    Then the cursor is at display line 1 display column 5
    When I press "j"
    Then the cursor is at display line 2 display column 23

  Scenario: Doublebyte text navigation in response pane (mixed ASCII/doublebyte, wrap off)
    Given there is a response in the response pane from:
      """
      こんにちは world
      """
    And I am in the response pane
    And the cursor is at display line 0 display column 0
    When I press "w"
    Then the cursor moves to display line 0 display column 5
    When I press "w"
    Then the cursor moves to display line 0 display column 11

  Scenario: Long doublebyte text does not crash application (doublebyte text, wrap off)
    Given there is a response in the response pane from:
      """
      これはとても長い日本語のテキストです。プログラミングにおいて、文字エンコーディングは重要な概念です。特にダブルバイト文字を扱う際には、適切な処理が必要になります。このテストは、長い日本語テキストが正しくレンダリングされ、アプリケーションがクラッシュしないことを確認するためのものです。
      """
    And I am in the response pane
    And the cursor is at display line 0 display column 0
    When I press "w"
    Then the terminal state should be valid
    When I press "l" 50 times
    Then the terminal state should be valid
    When I press "h" 20 times
    Then the terminal state should be valid
    And the response pane should display content

  Scenario: Mixed ASCII and doublebyte text handles navigation properly (mixed ASCII/doublebyte, wrap off)
    Given there is a response in the response pane from:
      """
      Response status: 200 OK
      
      {"message": "こんにちは、世界！", "status": "成功", "data": [{"name": "田中太郎", "age": 25}]}
      """
    And I am in the response pane  
    And the cursor is at display line 0 display column 0
    When I press "j" 3 times
    Then the cursor is at display line 3 display column 0
    When I press "w" 5 times
    Then the terminal state should be valid
    And the cursor position should be valid