Feature: Word wrap mode toggle functionality
  As a developer
  I want wrap mode toggle commands (:set wrap/:set nowrap) to work properly
  So that I can control how long lines are displayed

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane
    And the pane width is set to 112

  Scenario: Enable wrap mode resets horizontal scroll with double-byte characters
    Given I am in Insert mode
    # Type exactly 54 double-byte characters ending with し - triggers horizontal scrolling
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてとし" 
    And I press Escape
    # Go to end - this should trigger horizontal scrolling
    When I press "$"
    Then the cursor should be visible
    And I should see "し" in the output
    # Now enable wrap mode - horizontal scroll should be reset
    When I press ":"
    And I type "set wrap"
    And I press Enter
    # After wrap mode enabled, horizontal scroll should be reset
    # Line should start from beginning again
    Then the line starts with "あ"
    # Content should now wrap to multiple lines instead of horizontal scrolling
    And I should see "し" in the output
    And the cursor should be visible

  Scenario: Toggle back to nowrap mode restores horizontal scrolling
    Given I am in Insert mode
    # Create long line with double-byte characters
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてと"
    And I press Escape
    # Enable wrap mode first
    When I press ":"
    And I type "set wrap"
    And I press Enter
    Then the line starts with "あ"
    And the cursor should be visible
    # Now toggle back to nowrap mode
    When I press ":"
    And I type "set nowrap"  
    And I press Enter
    # Should work correctly in nowrap mode with horizontal scrolling
    When I press "0"
    When I press "$"
    Then the cursor should be visible
    And I should see "と" in the output

  Scenario: Wrap mode with mixed ASCII and double-byte characters
    Given I am in Insert mode
    # Create line with mixed content that exceeds pane width
    When I type "Start こんにちは世界 Middle あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん End"
    And I press Escape
    # In nowrap mode, should trigger horizontal scrolling
    When I press "$"
    Then the cursor should be visible
    # Enable wrap mode
    When I press ":"
    And I type "set wrap"
    And I press Enter
    # Should reset to beginning and wrap content
    Then the line starts with "Start"
    And I should see "End" in the output
    And the cursor should be visible

  Scenario: Wrap mode toggle preserves cursor logical position
    Given I am in Insert mode
    # Type long content
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそ"
    And I press Escape
    # Position cursor at specific character
    When I press "0"
    And I press "l" 20 times
    # Toggle wrap mode - cursor should stay at same logical position
    When I press ":"
    And I type "set wrap"
    And I press Enter
    Then the cursor should be visible
    # Verify by checking character navigation still works
    When I press "l"
    Then the cursor should be visible