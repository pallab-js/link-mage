# User Acceptance Testing (UAT) Checklist

## Test Session Information
**Date**: _______________  
**Tester**: _______________  
**Version**: Link Mage 0.1.0 Enhanced  
**Status**: ⏳ In Progress / ✅ Pass / ❌ Fail

---

## Pre-Test Setup

### Environment Check
- [ ] Application launches successfully
- [ ] No console errors on startup
- [ ] UI renders correctly
- [ ] All menus and panels accessible

### Initial State
- [ ] Graph canvas is empty (or shows saved graph)
- [ ] Sidebar is visible and functional
- [ ] Status bar shows "idle"

---

## Feature 1: Data Provenance Tracking

### Test 1.1: Manual Node Creation
- [ ] Add Entity panel is visible
- [ ] Can select "Domain" type
- [ ] Can enter value "example.com"
- [ ] Node appears on graph after clicking "Add Node"
- [ ] Node is selectable

**Provenance Check**:
- [ ] Select the domain node
- [ ] Provenance panel is visible in Sidebar
- [ ] Shows "Source: manual"
- [ ] Shows "Created: [timestamp]"
- [ ] Shows "By: user" (or system)
- [ ] Confidence bar shows 100% (green)
- [ ] Transform chain is empty

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 1.2: Transform Execution
- [ ] Right-click on domain node works
- [ ] Context menu appears
- [ ] "DNS Lookup" transform is available
- [ ] Click DNS Lookup executes
- [ ] Status shows "Running DNS Lookup..."
- [ ] New IP node(s) appear
- [ ] Edges connect domain to IP(s)
- [ ] Status shows success message

**Provenance Check (IP Node)**:
- [ ] Select an IP node
- [ ] Provenance shows "Source: dns_lookup"
- [ ] Transform chain shows "DNS Lookup" step
- [ ] Timestamp recorded
- [ ] Confidence ~95% (yellow-green)

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 1.3: Chain Transforms
- [ ] Right-click on IP node
- [ ] "GeoIP Lookup" is available
- [ ] Executes successfully
- [ ] Location node appears
- [ ] Edge connects IP to Location

**Provenance Check (Location Node)**:
- [ ] Select Location node
- [ ] Source: geoip_lookup
- [ ] Transform chain shows 2 steps
- [ ] Confidence ~85% (decayed)
- [ ] Both steps have timestamps

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Feature 2: Tags & Annotations

### Test 2.1: Create Tags
- [ ] "Tags & Notes" section visible in Sidebar
- [ ] Can expand/collapse section
- [ ] Tag input field works
- [ ] Enter "High Priority" as name
- [ ] Can select red color
- [ ] Click "+ Tag" creates tag
- [ ] Tag appears in tag list
- [ ] Usage count shows "0"

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 2.2: Apply Tags
- [ ] Select any node
- [ ] Tag list shows "+" button
- [ ] Click "+" applies tag
- [ ] Tag appears in "Applied to Selection"
- [ ] Usage count increments to "1"

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 2.3: Create Annotations
- [ ] With node selected, Annotations section visible
- [ ] Text area accepts input
- [ ] Type "Important finding - verified"
- [ ] Select yellow color
- [ ] Click "Add"
- [ ] Annotation appears with yellow bar
- [ ] Timestamp shown
- [ ] Pin button visible (📍)

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 2.4: Pin Annotation
- [ ] Hover over annotation
- [ ] Actions appear (pin, delete)
- [ ] Click pin (📍)
- [ ] Annotation becomes pinned (📌)
- [ ] Visual highlight appears
- [ ] Click again to unpin

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 2.5: Remove Tag
- [ ] With tagged node selected
- [ ] Click "X" on applied tag
- [ ] Tag removed from "Applied to Selection"
- [ ] Usage count decrements

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Feature 3: Transform Pipelines

### Test 3.1: View Pipelines
- [ ] Create new domain node (e.g., "google.com")
- [ ] Right-click on domain
- [ ] Context menu shows pipeline options
- [ ] "Domain Enrichment" or similar visible

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 3.2: Execute Pipeline
- [ ] Select pipeline from menu
- [ ] Status shows execution message
- [ ] Multiple transforms run sequentially
- [ ] Multiple new nodes appear (IPs, Locations, etc.)
- [ ] Edges created appropriately
- [ ] Completion message shown

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 3.3: Check Pipeline Results
- [ ] Select one of the new nodes
- [ ] Check provenance
- [ ] Should show pipeline in transform chain
- [ ] Multiple steps recorded

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Feature 4: Keyboard Shortcuts

### Test 4.1: Open Help
- [ ] Press `?` or `Shift+/`
- [ ] Help modal appears
- [ ] Shows all shortcut categories
- [ ] Platform-correct keys displayed (Ctrl vs ⌘)
- [ ] Click outside closes modal
- [ ] Press Escape closes modal

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 4.2: File Shortcuts
- [ ] Press `Ctrl+N` (or `⌘N`)
- [ ] Confirms new graph (or creates)
- [ ] Press `Ctrl+S` (or `⌘S`)
- [ ] Save dialog appears (if first save)

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 4.3: View Shortcuts
- [ ] Add some nodes to graph
- [ ] Press `Ctrl+F` (or `⌘F`)
- [ ] Graph fits to viewport
- [ ] Press `Ctrl++` (or `⌘+`)
- [ ] Zooms in
- [ ] Press `Ctrl+-` (or `⌘-`)
- [ ] Zooms out
- [ ] Press `Ctrl+L` (or `⌘L`)
- [ ] Auto-layout runs

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 4.4: Selection Shortcuts
- [ ] Press `Ctrl+A` (or `⌘A`)
- [ ] All nodes selected (highlighted)
- [ ] Press `Escape`
- [ ] All nodes deselected

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 4.5: Click Hint
- [ ] Click shortcut hint at bottom-right
- [ ] Help modal opens

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Feature 5: Advanced Filtering

### Test 5.1: Quick Presets
- [ ] Open Filter Panel
- [ ] Click "🎯 High Confidence"
- [ ] Graph filters (may not see change if all high confidence)
- [ ] Click "⭐ Hub Nodes"
- [ ] Filters apply
- [ ] Click "👁️ Hide Isolated"
- [ ] Isolated nodes hidden
- [ ] Click "🕐 Last 7 Days"
- [ ] Date filter applies

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 5.2: Type Filter
- [ ] Clear all filters (Reset button)
- [ ] Click 2-3 type chips (e.g., "IP", "Domain")
- [ ] Only selected types visible
- [ ] Active chips highlighted
- [ ] Click again to deselect
- [ ] All types visible again

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 5.3: Degree Range
- [ ] Adjust "Min degree" slider to 2
- [ ] Nodes with <2 connections hidden
- [ ] Adjust "Max degree" slider to 10
- [ ] Nodes with >10 connections hidden
- [ ] Reset to 0-999

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 5.4: Search
- [ ] Type in search box (e.g., "example")
- [ ] Wait ~300ms
- [ ] Graph filters to matching nodes
- [ ] Type different text
- [ ] Filters update after delay
- [ ] Clear search
- [ ] All nodes visible

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 5.5: Combined Filters
- [ ] Select types + set degree range + search
- [ ] All filters apply together
- [ ] Click "Reset All Filters"
- [ ] All filters clear
- [ ] Full graph visible

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Feature 6: Graph Statistics

### Test 6.1: View Statistics
- [ ] Stats Panel visible (may need to add to Sidebar)
- [ ] Shows total nodes count (correct)
- [ ] Shows total edges count (correct)
- [ ] Shows density percentage
- [ ] Shows connected components

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 6.2: Degree Stats
- [ ] Shows average degree
- [ ] Shows max degree
- [ ] Shows min degree
- [ ] Values are accurate

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 6.3: Special Nodes
- [ ] Shows isolated nodes count
- [ ] Shows hub nodes count (5+ connections)
- [ ] Counts are accurate

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 6.4: Type Breakdown
- [ ] Shows all node types
- [ ] Visual bars show proportions
- [ ] Counts match actual nodes

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 6.5: Top Hubs
- [ ] If hubs exist, shows top 5
- [ ] Degree numbers accurate
- [ ] Node labels display

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

### Test 6.6: Copy Stats
- [ ] Click copy button (📋)
- [ ] Paste in text editor
- [ ] Formatted statistics appear

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Integration Test: Full Workflow

### Scenario: Complete Investigation
- [ ] Create new graph (`Ctrl+N`)
- [ ] Add seed domain manually
- [ ] Run "Domain Enrichment" pipeline
- [ ] Tag important nodes (2-3 tags)
- [ ] Add annotations to key findings
- [ ] Filter to show only high-confidence nodes
- [ ] Check statistics
- [ ] Save graph (`Ctrl+S`)
- [ ] Close and reload application
- [ ] Load saved graph
- [ ] Verify nodes present
- [ ] Verify tags present (if persistence implemented)
- [ ] Verify annotations present (if persistence implemented)
- [ ] Verify provenance intact

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Performance Testing

### Large Graph
- [ ] Add 20+ nodes
- [ ] Run multiple transforms
- [ ] Graph remains responsive
- [ ] Filtering is fast (<1s)
- [ ] Layout completes successfully

**Result**: ⏳ / ✅ / ❌  
**Notes**: _________________________________________

---

## Issues Found

| ID | Feature | Issue Description | Severity | Steps to Reproduce |
|----|---------|-------------------|----------|-------------------|
| 1  |         |                   |          |                   |
| 2  |         |                   |          |                   |
| 3  |         |                   |          |                   |

**Severity**: 🔴 Critical / 🟠 High / 🟡 Medium / 🟢 Low

---

## Overall Assessment

### Pass Criteria
- [ ] All critical features work (Provenance, Tags, Pipelines)
- [ ] No crashes or data loss
- [ ] UI is responsive
- [ ] Keyboard shortcuts functional
- [ ] Documentation accurate

### Summary
**Total Tests**: _____  
**Passed**: _____  
**Failed**: _____  
**Skipped**: _____  

**Pass Rate**: _____%

### Recommendation
- [ ] ✅ **APPROVED** for production
- [ ] ⚠️ **APPROVED WITH MINOR ISSUES** (see above)
- [ ] ❌ **NOT APPROVED** - requires fixes

### Additional Comments
_________________________________________________
_________________________________________________
_________________________________________________

---

**Tester Signature**: _________________  
**Date**: _________________

**Reviewed By**: _________________  
**Date**: _________________
