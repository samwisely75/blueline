Feature: Issue #80 Regression - Dollar Sign with Exactly 53 Double-Byte Characters
  As a developer
  I need to ensure the $ command works correctly with exactly 53 double-byte characters
  So that the cursor positions at the last character and scrolls if needed

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Dollar sign with exactly 53 double-byte characters
    Given I am in Insert mode
    # Type exactly 53 double-byte characters ending with さし
    # 50 chars: あいうえおかきくけこ repeated 5 times
    # Plus 3 more: さしす
    When I type "あいうえおかきくけこあいうえおかきくけこあいうえおかきくけこあいうえおかきくけこあいうえおかきくけこさし"
    And I press Escape
    Then I should be in Normal mode
    When I press "0"
    Then the cursor is at display line 0 display column 0
    When I press "$"
    # With 53 double-byte characters:
    # - Logical positions: 0-52 (53 positions)
    # - Last character (index 52) should be at display columns 104-105
    # - Cursor should be at the start of the last character
    Then the cursor is at display line 0 display column 104
    # Verify we can see the last character
    And I should see "し" in the output