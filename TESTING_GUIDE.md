# Link Mage - New Features Testing Guide

## Overview
This guide helps you test all the new features added to Link Mage. Each section includes step-by-step instructions and expected results.

---

## 1. Data Provenance Tracking 🎯

### What to Test:
- Provenance is automatically created when transforms run
- Confidence scores decrease with each transform step
- Provenance panel displays complete history

### Test Steps:

1. **Create a manual node**
   - Open Sidebar → Add Entity
   - Type: Domain, Value: `example.com`
   - Click "Add Node"
   - Select the node
   - **Expected**: Provenance panel shows "Source: manual", Confidence: 100%

2. **Run a transform**
   - Right-click the domain node
   - Select "DNS Lookup" from context menu
   - Wait for completion
   - Select one of the new IP nodes
   - **Expected**: 
     - Provenance shows "Source: dns_lookup"
     - Transform chain shows "DNS Lookup" step
     - Confidence: ~95%

3. **Run another transform on result**
   - Right-click an IP node
   - Select "GeoIP Lookup"
   - Select the new Location node
   - **Expected**:
     - Provenance shows "Source: geoip_lookup"
     - Transform chain shows 2 steps
     - Confidence: ~85% (decayed)

### Success Criteria:
✅ All nodes show provenance information
✅ Transform chain is accurate
✅ Confidence decreases with each step
✅ Timestamps are recorded

---

## 2. Tags & Annotations 🏷️

### What to Test:
- Create and manage tags
- Apply tags to nodes
- Add annotations with colors
- Pin important annotations

### Test Steps:

1. **Create tags**
   - Open Sidebar → "Tags & Notes" section (expand if collapsed)
   - Enter tag name: "High Priority"
   - Select red color
   - Click "+ Tag"
   - **Expected**: Tag appears in tag list with usage count "0"

2. **Apply tag to node**
   - Select any node
   - Click the "+" button next to your tag
   - **Expected**: Tag appears in "Applied to Selection" section

3. **Add annotation**
   - With node still selected
   - In Annotations section, type a note
   - Select a color (e.g., yellow)
   - Click "Add"
   - **Expected**: Annotation appears with colored bar

4. **Pin annotation**
   - Hover over annotation
   - Click pin icon (📍 → 📌)
   - **Expected**: Annotation gets highlighted border

5. **Remove tag**
   - With node selected, click "X" on applied tag
   - **Expected**: Tag removed, usage count decreases

### Success Criteria:
✅ Tags can be created with custom colors
✅ Tags apply to selected nodes
✅ Annotations display with color coding
✅ Pinned annotations are highlighted
✅ Usage counts update correctly

---

## 3. Transform Pipelines ⚙️

### What to Test:
- Built-in pipelines are available
- Pipelines execute multiple transforms automatically
- Results are tracked

### Test Steps:

1. **View available pipelines**
   - Right-click a Domain node
   - Look for pipeline options in context menu
   - **Expected**: See "Domain Enrichment" and other pipelines

2. **Run a pipeline**
   - Select "Domain Enrichment" pipeline
   - Watch execution progress (if UI implemented)
   - Wait for completion
   - **Expected**: 
     - Multiple new nodes created (IPs, Locations, etc.)
     - Status message shows results

3. **Check results**
   - Count new nodes
   - Check their provenance
   - **Expected**: All nodes show pipeline in provenance chain

### Success Criteria:
✅ Pipelines appear in context menu
✅ Multiple transforms execute in sequence
✅ All results are added to graph
✅ Provenance tracks pipeline execution

---

## 4. Keyboard Shortcuts ⌨️

### What to Test:
- Help modal opens with `?`
- Shortcuts work as documented
- Platform-specific keys display correctly

### Test Steps:

1. **Open help**
   - Press `?` or `Shift+/`
   - **Expected**: Modal opens showing all shortcuts

2. **Test file operations**
   - Press `Ctrl+N` (or `⌘N` on Mac)
   - **Expected**: Confirms new graph (or creates if no unsaved changes)

3. **Test view shortcuts**
   - Press `Ctrl+F` → Graph fits to viewport
   - Press `Ctrl++` → Zooms in
   - Press `Ctrl+-` → Zooms out
   - Press `Ctrl+L` → Auto layout runs

4. **Test selection**
   - Select a node
   - Press `Ctrl+A` → All nodes selected
   - Press `Escape` → Deselect all

5. **Click hint**
   - Click the shortcut hint at bottom-right
   - **Expected**: Help modal opens

### Success Criteria:
✅ Help modal displays all 60+ shortcuts
✅ Shortcuts execute correctly
✅ Platform keys show correctly (Ctrl vs ⌘)
✅ Modal closes on click outside or Escape

---

## 5. Advanced Filtering 🔍

### What to Test:
- Filter by multiple criteria
- Quick presets work
- Search is debounced
- Filters combine correctly

### Test Steps:

1. **Use quick presets**
   - Open Filter Panel
   - Click "🎯 High Confidence"
   - **Expected**: Only high-confidence nodes visible

2. **Filter by type**
   - Click on 2-3 node type chips
   - **Expected**: Only those types visible

3. **Filter by degree**
   - Adjust "Min degree" slider to 2
   - **Expected**: Nodes with <2 connections hidden

4. **Search**
   - Type in search box
   - Wait 300ms
   - **Expected**: Graph filters after delay (not on every keystroke)

5. **Combine filters**
   - Select types + set degree range + search
   - **Expected**: All filters apply together

6. **Reset**
   - Click "Reset All Filters"
   - **Expected**: All filters clear, full graph visible

### Success Criteria:
✅ Presets apply correct filters
✅ Multiple filters combine properly
✅ Search is debounced (no lag)
✅ Reset clears all filters
✅ Edges update when nodes filtered

---

## 6. Graph Statistics 📊

### What to Test:
- Statistics calculate correctly
- Updates when graph changes
- Top hubs identified
- Copy to clipboard works

### Test Steps:

1. **View statistics**
   - Add several nodes of different types
   - Add some edges
   - Open Stats Panel (if integrated in Sidebar)
   - **Expected**: 
     - Node/edge counts correct
     - Types breakdown shows all types
     - Degree stats calculated

2. **Add more nodes**
   - Add 5 more nodes
   - **Expected**: Stats update automatically

3. **Check hubs**
   - Create a node with many connections (5+)
   - **Expected**: Node appears in "Top Hubs" section

4. **Copy stats**
   - Click copy button
   - Paste somewhere
   - **Expected**: Formatted statistics text copied

### Success Criteria:
✅ All statistics calculate correctly
✅ Stats update when graph changes
✅ Top hubs identified accurately
✅ Copy to clipboard works
✅ Density formula correct

---

## Integration Tests

### Full Workflow Test:

1. Create new graph
2. Add domain node manually
3. Run "Domain Enrichment" pipeline
4. Tag important nodes
5. Add annotations to key findings
6. Filter to show only high-confidence nodes
7. Check statistics
8. Save graph
9. Reload graph
10. Verify all data preserved (tags, annotations, provenance)

### Expected Result:
All features work together seamlessly without errors.

---

## Known Limitations

1. **Annotations persistence**: Currently frontend-only, may not persist after reload
2. **Pipeline timeout**: Simplified implementation, actual timeout requires async
3. **Tag persistence**: Tags stored in frontend, need backend integration for persistence

---

## Reporting Issues

For any issues found during testing, please note:
- Steps to reproduce
- Expected vs actual behavior
- Browser/OS version
- Console errors (if any)

---

**Last Updated**: April 1, 2026
**Version**: Link Mage 0.1.0 (Enhanced)
