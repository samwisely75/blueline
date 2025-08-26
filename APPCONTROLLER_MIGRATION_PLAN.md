# AppController Migration Plan - Updated

## Current Status After Consolidation

### Feature Files (22 total)
We've consolidated from 35 to 22 feature files, significantly simplifying the migration work.

### Already Migrated (5 features) ✅
1. `application_lifecycle.feature`
2. `mode_transitions.feature` 
3. `command_line.feature`
4. `arrow_keys_all_modes.feature` (removed - consolidated into cursor_navigation)
5. `text_insert_mode.feature` (removed - consolidated into text_editing)

### New Consolidated Features to Migrate (6 features) 🆕
These are the new consolidated files we created that need AppController migration:

1. **`cursor_navigation.feature`** - Comprehensive cursor movement (replaces 5 files)
2. **`normal_mode_commands.feature`** - All normal mode editing commands (replaces 3 files)
3. **`text_editing.feature`** - Text manipulation operations (replaces 4 files)
4. **`visual_modes.feature`** - All visual mode operations (replaces 3 files)
5. **`yank_paste.feature`** - Yank and paste operations (replaces yank.feature)
6. **`unicode_i18n.feature`** - International text support (replaces 2 files)

### Remaining Original Features (11 features) 📝
1. `cursor_doublebyte_edge.feature` - Edge cases for cursor with double-byte chars
2. `cursor_flicker_fix.feature` - Specific cursor flicker regression test
3. `horizontal_scrolling.feature` - Horizontal scroll behavior
4. `http_request_flow.feature` - HTTP request/response functionality
5. `issue_190_visual_delete_multibyte.feature` - Specific regression test
6. `line_number_toggle.feature` - Line number display toggle
7. `response_navigation.feature` - Response pane navigation
8. `tab_handling.feature` - Tab key behavior
9. `text_append_doublebyte.feature` - Append with double-byte characters
10. `text_deletion.feature` - Text deletion operations
11. `window.feature` - Window/pane management
12. `wrap_mode.feature` - Text wrapping behavior
13. `wrap_syntax.feature` - Wrap mode syntax

## Proposed Issue Reorganization

### Close These Issues (Already Done via Consolidation)
- **#206** - Normal Mode Commands Group ✅ (consolidated into `normal_mode_commands.feature`)

### Update These Issues

#### Issue #207: Text Manipulation Group
**Update to include:**
- [ ] `text_editing.feature` (NEW - consolidated)
- [ ] `text_deletion.feature` 
- [ ] `text_append_doublebyte.feature`
- [ ] `tab_handling.feature`

#### Issue #208: Visual Mode Group  
**Update to include:**
- [ ] `visual_modes.feature` (NEW - consolidated)
- [ ] `issue_190_visual_delete_multibyte.feature` (regression test)

#### Issue #209: Navigation Group
**Update to include:**
- [ ] `cursor_navigation.feature` (NEW - consolidated)
- [ ] `cursor_doublebyte_edge.feature` (edge cases)
- [ ] `cursor_flicker_fix.feature` (regression test)
- [ ] `response_navigation.feature`

#### Issue #210: Display/Rendering Group
**Keep as is:**
- [ ] `horizontal_scrolling.feature`
- [ ] `wrap_mode.feature`
- [ ] `wrap_syntax.feature`
- [ ] `line_number_toggle.feature`
- [ ] `window.feature`

#### Issue #211: Advanced Features Group
**Update to include:**
- [ ] `http_request_flow.feature`
- [ ] `unicode_i18n.feature` (NEW - consolidated)
- [ ] `yank_paste.feature` (NEW - consolidated)

## Migration Strategy

### Phase 1: Core Functionality (Priority High)
1. **Text Editing** (#207) - Foundation for all editing operations
   - Start with `text_editing.feature` as it's well-organized with Scenario Outlines
   - Then `text_deletion.feature`, `tab_handling.feature`

2. **Navigation** (#209) - Essential for cursor movement
   - Start with `cursor_navigation.feature` 
   - Edge cases can wait

3. **Normal Mode Commands** (Already closed #206)
   - `normal_mode_commands.feature` - Core vim commands

### Phase 2: Visual and Yank/Paste (Priority Medium)
4. **Visual Modes** (#208)
   - `visual_modes.feature` - Visual selection operations

5. **Yank/Paste** (Part of #211)
   - `yank_paste.feature` - Copy/paste operations

### Phase 3: Display and Advanced (Priority Lower)
6. **Display/Rendering** (#210)
   - Can be done incrementally as these are mostly UI concerns

7. **Advanced Features** (#211)
   - HTTP and Unicode support

## Benefits of This Approach

1. **Reduced Complexity**: Only 22 files to migrate instead of 35
2. **Better Organization**: Consolidated files use Scenario Outlines, reducing duplication
3. **Faster Testing**: Tests already run in ~2.5 minutes with parallelization
4. **Clear Priorities**: Core editing/navigation first, advanced features later
5. **Regression Tests Preserved**: Keep specific issue regression tests separate

## Next Steps

1. Update GitHub issues #207-211 with new file lists
2. Close #206 as it's been consolidated
3. Start migration with `text_editing.feature` as the first target
4. Use the patterns we established in the already-migrated features