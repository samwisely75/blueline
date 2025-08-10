Feature: Cursor Navigation with Double-Byte Characters
  As a user working with international content
  I want cursor navigation commands to work correctly with double-byte characters
  So that I can efficiently edit text in any language

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Dollar sign with long line of double-byte characters triggers horizontal scroll
    Given I am in Insert mode
    # Create a line with 55 double-byte Japanese characters (110 display columns)
    # The pane width is typically 112 columns, so this should fit exactly or nearly
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこ"
    And I press Escape
    And I press "0"
    # At this point cursor is at position 0, no scrolling needed
    When I press "$"
    # The cursor should move to the last character position (54th character, 0-indexed)
    # Display column should be 108 (54 chars * 2 width each = 108)
    Then the cursor is at display line 0 display column 108
    And I should see "こ" in the output

  Scenario: Dollar sign with extra-long double-byte line requires horizontal scroll
    Given I am in Insert mode
    # Create a line with 60 double-byte characters (120 display columns)
    # This exceeds typical pane width of 112 columns
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそ"
    And I press Escape
    And I press "0"
    # Now the beginning of the line is visible, end is not
    When I press "$"
    # The cursor should move to the last character (59th character, 0-indexed)
    # Display column should be 118 (59 chars * 2 width each = 118)
    # This should trigger horizontal scrolling since it exceeds pane width
    Then the cursor is at display line 0 display column 118
    And I should see "そ" in the output

  Scenario: Navigation from dollar position with h moves correctly
    Given I am in Insert mode
    # Create line with 60 double-byte characters
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそ"
    And I press Escape
    And I press "$"
    # Now at the last character "そ" at display column 118
    When I press "h"
    # Should move to "せ" at display column 116
    Then the cursor is at display line 0 display column 116
    When I press "h"
    # Should move to "す" at display column 114
    Then the cursor is at display line 0 display column 114

  Scenario: Dollar sign with mixed ASCII and double-byte characters
    Given I am in Insert mode
    # Create a mixed line that exceeds pane width
    When I type "Hello World これは長い日本語のテキストです。とても長い文章なので画面の幅を超えてしまいます。最後の文字まで正しく移動できるはずです。"
    And I press Escape
    And I press "0"
    When I press "$"
    # Should be at the last character "。"
    # The mixed text has 12 ASCII chars + many double-byte chars
    Then I should see "。" in the output