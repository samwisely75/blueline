# Page Scrolling Analysis and Options

## Current Issue Summary

### Problem Description
Ctrl+F page scrolling has incorrect cursor positioning behavior:

**Expected behavior:**
- Start: `RESPONSE 1:6 (1:6)` - cursor at logical column 6, display position (1,6)
- After Ctrl+F: `RESPONSE 1:1050 (1:6)` - cursor advances to logical column 1050, stays at display position (1,6)
- Content scrolls, cursor visually stays in same spot

**Actual behavior:**
- Start: `RESPONSE 1:6 (1:6)`
- After Ctrl+F: `RESPONSE 1:2094 (19:6)` - cursor jumps to logical column 2094, display position (19,6)
- Cursor moves both logically AND visually (wrong!)

### Root Cause Analysis

1. **Logical position calculation wrong**: Advances by ~2088 characters instead of ~1044
2. **Display position calculation wrong**: Cursor moves to display row 19 instead of staying at row 1
3. **Architecture issues**: Feature envy between PaneState and DisplayCache, inconsistent types (Position vs LogicalPosition)

### Debug Evidence
```
scroll_vertically_by_page: viewport moved from 7 to 25, cursor moved from (0, 817) to (0, 2905)
```
- Viewport moves correctly (18 display lines)
- Cursor logical position jumps by 2088 characters (too much)

## Attempted Fixes

### Fix 1: Corrected Page Scrolling Logic (Partial Success)
- **Changed**: Move viewport by `pane_dimensions.height` instead of massive character jumps
- **Added**: Status bar update events (`StatusBarUpdateRequired`)
- **Fixed**: Type conversion issues between `Position` and `LogicalPosition`
- **Result**: Viewport scrolling works, but cursor positioning still incorrect

### Fix 2: Improved Column Calculation (No Change)
- **Changed**: Use `get_display_line()` to clamp column within target line
- **Added**: Better debug logging
- **Result**: No improvement, cursor still jumps incorrectly

## Architecture Problems Identified

### Current Issues
1. **Feature Envy**: `PaneState::scroll_vertically_by_page()` heavily manipulates DisplayCache methods
2. **Split Responsibilities**: Navigation logic scattered between PaneState and DisplayCache  
3. **Type Inconsistency**: Methods claim to return `LogicalPosition` but return `Position`
4. **Unclear Ownership**: Who owns cursor movement? PaneState or DisplayCache?

### Recommended Consolidation
Move all navigation logic to PaneState (Option 2 from architectural discussion):
- PaneState owns: cursor, buffer, scroll offset, viewport management
- DisplayCache provides: coordinate conversion utilities only
- Benefits: Clear ownership, consistent types, less feature envy

## Navigation System Design Options

### Option A: Content-Based Page Scrolling
```rust
fn page_scroll(&mut self, direction: Direction) -> NavigationResult {
    let chars_per_page = self.estimate_chars_per_page();
    let new_logical_pos = if direction == Down {
        self.cursor.logical.advance_by(chars_per_page)
    } else {
        self.cursor.logical.retreat_by(chars_per_page)  
    };
    
    self.move_to_logical_position(new_logical_pos);
    self.viewport.ensure_cursor_visible();
}
```

**Pros:** Simple character-based movement
**Cons:** Doesn't match traditional editor behavior

### Option B: Display-Based Page Scrolling (RECOMMENDED)
```rust
fn page_scroll(&mut self, direction: Direction) -> NavigationResult {
    let page_height = self.viewport.height - 2; // Leave overlap
    
    // Move viewport
    let new_viewport_start = if direction == Down {
        self.viewport.start + page_height
    } else {
        self.viewport.start.saturating_sub(page_height)
    };
    
    self.viewport.move_to(new_viewport_start);
    
    // Keep cursor at same relative position within viewport
    let cursor_relative_pos = self.cursor.relative_to_viewport();
    let new_logical_pos = self.display_mapper.viewport_relative_to_logical(
        cursor_relative_pos
    );
    
    self.cursor.move_to(new_logical_pos);
}
```

**Pros:** Traditional behavior (vim, less), predictable, cursor stays visually fixed
**Cons:** Slightly more complex viewport management

## Ideal Clean Architecture

```
┌─────────────────┐
│ NavigationEngine│ ← High-level navigation commands
├─────────────────┤
│ ViewportManager │ ← Manages what's visible, scroll offset  
├─────────────────┤
│ DisplayMapper   │ ← Converts logical ↔ display coordinates
├─────────────────┤
│ ContentBuffer   │ ← Raw content, logical positions
└─────────────────┘
```

### Benefits
1. **Clear ownership**: NavigationEngine owns all movement
2. **Predictable page scrolling**: Viewport-based, cursor stays visually fixed
3. **Separation of concerns**: Viewport, cursor, and display mapping are separate
4. **Easy testing**: Each component can be tested independently
5. **No type confusion**: One cursor position type with clear semantics

## Next Steps

1. **Commit current state** as working baseline with known issues
2. **Try Option B implementation** within existing PaneState structure
3. **Test behavior** against expected results
4. **Consider full architectural refactor** if Option B proves the approach works

## Test Cases

### Sample Data
Response pane contains this JSON data (wrapped across multiple display lines):
```
  1 {"took":210,"timed_out":false,"num_reduce_phases":2,"_shards":{"total":542,"successful":542,"skipped":0,"failed":0},
    "hits":{"total":{"value":10000,"relation":"gte"},"max_score":1.0,"hits":[{"_index":".asset-criticality.asset-critica
    lity-default","_id":"service.name:metricbeat","_score":1.0,"_source":{"id_field":"service.name","@timestamp":"2025-0
    7-30T02:24:42.895Z","criticality_level":"extreme_impact","service":{"name":"metricbeat","asset":{"criticality":"extr
    eme_impact"}},"asset":{"criticality":"extreme_impact"},"event":{"ingested":"2025-07-30T02:24:42.899873087Z"},"id_val
    ue":"metricbeat"}},{"_index":".asset-criticality.asset-criticality-default","_id":"user.name:satoshi.iizuka","_score
    ":1.0,"_source":{"id_field":"user.name","@timestamp":"2025-07-30T02:22:36.245Z","criticality_level":"medium_impact",
    "asset":{"criticality":"medium_impact"},"event":{"ingested":"2025-07-30T02:22:36.248487946Z"},"user":{"name":"satosh
    i.iizuka","asset":{"criticality":"medium_impact"}},"id_value":"satoshi.iizuka"}},{"_index":".ds-.monitoring-beats-8-
    mb-2025.08.01-000051","_id":"aY2WdJgBVKdRCT3-MqnJ","_score":1.0,"_source":{"@timestamp":"2025-08-04T10:17:27.221Z","
    agent":{"ephemeral_id":"6bb9299f-ea51-4105-80ef-c79200ff9f1c","id":"24374d4b-5e8b-4ffc-8c00-d9a7e19417b3","name":"f6
    fcaf90ea48","type":"metricbeat","version":"8.19.0"},"ecs":{"version":"8.0.0"},"host":{"architecture":"x86_64","hostn
    ame":"f6fcaf90ea48","name":"f6fcaf90ea48"},"event":{"duration":2645278,"dataset":"beat.state","module":"beat"},"metr
    icset":{"name":"state","period":10000},"service":{"address":"http://localhost:6791/processes/apm-server-es-container
    host/state","type":"beat"},"beat":{"elasticsearch":{"cluster":{"id":"AmtcpDCgQ3Wu5tSwtaAL0Q"}},"state":{"beat":{"nam
    e":"f6fcaf90ea48","host":"f6fcaf90ea48","type":"apm-server","uuid":"80d7ab90-6edc-4462-9b38-e48eadf00292","version":
    "8.19.0"},"management":{"enabled":true},"service":{"id":"80d7ab90-6edc-4462-9b38-e48eadf00292","name":"apm-server","
    version":"8.19.0"},"output":{"name":"elasticsearch"},"host":{"os":{"kernel":"5.15.0-1032-gcp","name":"Wolfi","platfo
    rm":"wolfi","version":"20230201"}},"cluster":{"uuid":"AmtcpDCgQ3Wu5tSwtaAL0Q"}}}}},{"_index":".ds-.monitoring-beats-
    8-mb-2025.08.01-000051","_id":"a42WdJgBVKdRCT3-MqnJ","_score":1.0,"_source":{"@timestamp":"2025-08-04T10:17:27.227Z"
    ,"event":{"dataset":"beat.stats","module":"beat","duration":5062218},"metricset":{"name":"stats","period":10000},"be
    at":{"elasticsearch":{"cluster":{"id":"AmtcpDCgQ3Wu5tSwtaAL0Q"}},"stats":{"apm_server":{"agentcfg":{"elasticsearch":
    {"cache":{"entries":{"count":0},"refresh":{"successes":12446}}}},"root":{"request":{"count":74225},"response":{"coun
    t":74225,"valid":{"count":74225,"ok":74225}}}},"uptime":{"ms":373351716},"runtime":{"goroutines":85},"output":{"elas
    ticsearch":{"indexers":{"created":0,"destroyed":0,"active":1},"bulk_requests":{"completed":0,"available":11}}},"cpu"
    :{"total":{"ticks":807110,"time":{"ms":807110},"value":807110},"user":{"ticks":607700,"time":{"ms":607700}},"system"
    :{"ticks":199410,"time":{"ms":199410}}},"libbeat":{"output":{"type":"elasticsearch"}},"cgroup":{"cpu":{"stats":{"per
    iods":988698,"throttled":{"periods":0,"ns":0}},"cfs":{"quota":{"us":800000},"period":{"us":100000}},"id":"/"},"cpuac
    ct":{"id":"/","total":{"ns":4.031967328484e+12}},"memory":{"id":"/","mem":{"limit":{"bytes":1.073741824e+09},"usage"
    :{"bytes":4.06818816e+08}}}},"memstats":{"gc_next":16224847,"memory":{"alloc":11341320,"total":75094054264},"rss":34
    861056},"system":{"cpu":{"cores":32},"load":{"norm":{"1":0.1138,"15":0.0572,"5":0.0688},"1":3.64,"15":1.83,"5":2.2}}
    ,"beat":{"type":"apm-server","uuid":"80d7ab90-6edc-4462-9b38-e48eadf00292","version":"8.19.0","name":"f6fcaf90ea48",
    "host":"f6fcaf90ea48"},"handles":{"limit":{"soft":1048576,"hard":1048576},"open":21},"info":{"ephemeral_id":"61c21b0
    3-e5cc-4446-a4b8-d4835d9ac532","name":"apm-server","uptime":{"ms":3.73351716e+08},"version":"8.19.0"}},"id":"80d7ab9
    0-6edc-4462-9b38-e48eadf00292","type":"apm-server"},"host":{"name":"f6fcaf90ea48"},"agent":{"ephemeral_id":"6bb9299f
    -ea51-4105-80ef-c79200ff9f1c","id":"24374d4b-5e8b-4ffc-8c00-d9a7e19417b3","name":"f6fcaf90ea48","type":"metricbeat",
```

### Expected Test Case
- **Setup**: Cursor at `RESPONSE 1:6 (1:6)`, JSON content wrapped to multiple display lines
- **Action**: Press Ctrl+F
- **Expected Results**:
  1. New logical position: `RESPONSE 1:1050 (1:6)` - advances ~1044 chars, stays at display (1,6)
  2. Response pane should start with line 19 containing `rm":"wolfi" ...`  
  3. Cursor should be at first line of response pane, in front of "wolfi"
- **Current Results**:
  1. New position: `RESPONSE 1:2094 (19:6)` - advances ~2088 chars, moves to display (19,6)
  2. Response pane starts with line 19 (correct)
  3. Cursor at bottom of line, in front of "-" (wrong position)

The key insight: **logical position should advance by page worth of content, but display position should stay visually the same**.