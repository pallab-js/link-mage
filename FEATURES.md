# Link Mage - New Features Documentation

## Table of Contents
1. [Data Provenance Tracking](#1-data-provenance-tracking)
2. [Annotations & Tagging](#2-annotations--tagging)
3. [Transform Machines](#3-transform-machines)
4. [Keyboard Shortcuts](#4-keyboard-shortcuts)
5. [Advanced Filtering](#5-advanced-filtering)

---

## 1. Data Provenance Tracking

### Overview
Every entity in Link Mage now tracks its origin and transformation history, providing forensic integrity for investigations.

### Features

#### Automatic Tracking
- **Source Detection**: Automatically records how each entity was created
- **Transform Chain**: Maintains complete history of transforms applied
- **Confidence Scoring**: Calculates reliability based on source and chain length
- **Timestamps**: Records creation and modification times

#### Supported Sources
| Source | Confidence | Description |
|--------|-----------|-------------|
| Manual | 100% | User-created entities |
| DNS Lookup | 95% | Direct DNS queries |
| WHOIS Lookup | 90% | Registry data |
| VirusTotal | 95% | Security vendor aggregation |
| GeoIP | 85% | Geolocation databases |
| Web Scrape | 75% | Scraped from webpages |
| Remote Transform | 70% | Third-party transforms |
| Plugin | 80% | Native plugins |
| Import | 85% | Imported from files |

#### Confidence Decay
Confidence decreases with each transform step:
- **Base confidence**: Determined by source type
- **Decay factor**: 2% per transform step
- **Success rate**: Failed transforms reduce confidence
- **Minimum**: 10% confidence floor

**Formula**: `confidence = base × (0.98^steps) × success_rate`

### Usage

#### Viewing Provenance
1. Select any node
2. Open "Provenance" section in Sidebar
3. View:
   - Source icon and name
   - Creation timestamp
   - Creator (user or system)
   - Confidence bar (color-coded)
   - Transform chain with status
   - External references

#### Confidence Colors
- 🟢 **Green** (80-100%): High confidence
- 🟡 **Yellow** (50-79%): Medium confidence  
- 🔴 **Red** (0-49%): Low confidence

### API Reference

```typescript
interface Provenance {
  source: ProvenanceSource;
  createdAt: string;        // ISO 8601
  createdBy: string;
  transformChain: TransformStep[];
  confidence: number;       // 0.0 - 1.0
  externalRef?: string;
  notes?: string;
}
```

---

## 2. Annotations & Tagging

### Overview
Add contextual notes and classification labels to entities for better organization and collaboration.

### Tags System

#### Creating Tags
1. Open "Tags & Notes" section
2. Enter tag name
3. Select color (10 presets available)
4. Click "+ Tag"

#### Tag Colors
- 🔴 Red, 🟠 Orange, 🟡 Yellow
- 🟢 Green, 🔵 Blue, 🟣 Purple
- 🩷 Pink, ⚫ Gray, + more

#### Applying Tags
1. Select target node(s) or edge(s)
2. Click "+" button on desired tag
3. Tag appears in "Applied to Selection"
4. Usage count increments

#### Managing Tags
- **Edit**: Click pencil icon, modify name/color, save
- **Delete**: Click trash icon, confirm deletion
- **Remove from entity**: Click "X" on applied tag

### Annotations

#### Creating Annotations
1. Select entity
2. Type note in text area
3. Select color (6 options)
4. Click "Add"

#### Annotation Features
- **Pinning**: Click pin icon to highlight important notes
- **Colors**: Yellow, Green, Blue, Purple, Red, Orange
- **Timestamps**: Automatically recorded
- **Editing**: Click to edit, save to update

#### Visual Indicators
- **Colored bar**: Left edge shows annotation color
- **Pinned border**: Highlighted border for pinned notes
- **Hover actions**: Delete and pin buttons appear on hover

### Best Practices

#### Tagging Strategy
```
🔴 Critical    - High-priority findings
🟠 Review      - Needs verification  
🟢 Verified    - Confirmed accurate
🔵 Note        - General information
🟣 Related     - Connected entities
```

#### Annotation Examples
- Investigation notes
- Evidence references
- Hypothesis documentation
- Collaboration comments
- External links

---

## 3. Transform Machines

### Overview
Machines chain built-in transforms in sequence, feeding each step's output
nodes into the next.

### Built-in Machines

#### Domain Footprint
**Purpose**: Comprehensive domain intelligence
**Steps**: DNS Lookup → WHOIS → Scrape Webpage

#### IP Intel
**Purpose**: Multi-angle IP investigation
**Steps**: GeoIP Lookup → WHOIS → VirusTotal Lookup

### Usage
1. Right-click a seed node
2. Choose a machine from the context menu
3. Results are added to the graph automatically
4. Per-step results and errors appear in the status bar

### Success Criteria
- Each step only runs on nodes it applies to
- Machine-step failures are reported without aborting the run
- New nodes carry the executed transforms in their provenance
---

## 4. Keyboard Shortcuts

### Overview
Power-user keyboard navigation for efficient investigations.

### Modifier Keys

| Key | Windows/Linux | macOS |
|-----|--------------|-------|
| Mod | Ctrl | ⌘ Command |
| Alt | Alt | ⌥ Option |
| Shift | Shift | ⇧ Shift |

### Essential Shortcuts

#### File Operations
```
Mod+N    New graph
Mod+O    Open graph
Mod+S    Save graph
Mod+⇧+S  Save as...
```

#### Navigation
```
Mod+F    Fit graph to viewport
Mod++    Zoom in
Mod+-    Zoom out
Mod+L    Auto layout
Mod+H    Cycle layout
```

#### Selection
```
Mod+A    Select all nodes
Escape   Deselect all
Delete   Delete selected
```

#### Quick Actions
```
?            Show keyboard help
Mod+⇧+F      Clear all filters
Mod+B        Toggle sidebar
Mod+,        Settings
```

### Customization

Shortcuts are defined in `KeyboardShortcuts.svelte`. To customize:

1. Open file in editor
2. Find shortcut in `handleKeydown()`
3. Modify key combination
4. Update help modal

---

## 5. Advanced Filtering

### Overview
Powerful filtering capabilities to focus on relevant entities in large graphs.

### Filter Types

#### 1. Entity Type Filter
- Click type chips to toggle
- Multiple types can be selected
- Visual indication of active filters

#### 2. Degree Range
- **Min degree**: Hide nodes with fewer connections
- **Max degree**: Hide highly-connected hubs
- Use sliders for quick adjustment

#### 3. Search
- Searches all properties
- Case-insensitive
- Matches type, label, value, any property
- Debounced (300ms delay)

#### 4. Provenance Source
- Filter by creation source
- Multiple sources can be selected
- Useful for verifying data quality

#### 5. Confidence Level
- Minimum confidence threshold
- Filter out low-confidence entities
- Slider or numeric input

#### 6. Date Range
- Filter by creation date
- From/To date pickers
- Useful for timeline analysis

### Quick Presets

| Preset | Description | Use Case |
|--------|-------------|----------|
| 🎯 High Confidence | Confidence ≥ 80% | Quality focus |
| ⭐ Hub Nodes | Degree ≥ 5 | Find key players |
| 👁️ Hide Isolated | Hide degree 0 | Clean up graph |
| 🕐 Last 7 Days | Recent entities | Fresh data only |

### Filter Logic

#### Standard Mode (Default)
- Selected filters **hide** matching entities
- Example: Select "IP" → IP nodes hidden

#### Invert Mode
- Selected filters **show** only matching entities
- Example: Select "IP" → Only IP nodes visible

### Combining Filters

Filters are **additive** - all active filters must match:

```
Visible = (Type matches) 
       AND (Degree in range) 
       AND (Search matches) 
       AND (Provenance matches) 
       AND (Confidence >= min)
```

### Performance

- **Debounced search**: 300ms delay prevents lag
- **Efficient filtering**: O(n) complexity
- **Edge cleanup**: Edges auto-hidden when nodes filtered

---


## Appendix

### File Locations

```
src/lib/
  types/
    provenance.ts      # Provenance types
  stores/
    annotations.ts     # Annotation/tag stores
    stores.ts          # Graph/selection/theme stores, machine types
  filterStore.ts       # Filtering logic
  components/
    ProvenancePanel.svelte
    TagManager.svelte
    ShortcutHelp.svelte
    FilterPanel.svelte

src-tauri/src/
  provenance.rs        # Rust provenance module
  transforms.rs        # Transforms & machines
```

### Configuration

#### Confidence Decay
Edit the decay factor in `src-tauri/src/provenance.rs`:
```rust
let decay_factor = 0.98;  // 2% per step
```

#### Hub Threshold
Edit the hub preset in `src/lib/filterStore.ts`:
```typescript
hubs: { minDegree: 5, name: 'Hub Nodes (5+ connections)' },
```

#### Search Debounce
Edit in `FilterPanel.svelte`:
```typescript
const SEARCH_DEBOUNCE_MS = 300;  // Milliseconds
```

### Troubleshooting

#### Provenance Not Showing
- Check node was created after implementation
- Verify transform executed successfully
- Check console for errors

#### Tags Not Persisting
- Currently frontend-only (known limitation)
- Backend integration needed for persistence

#### Filters Not Working
- Check if invert mode is enabled
- Verify filter criteria is set
- Try resetting all filters

---

**Version**: 0.1.0 Enhanced  
**Last Updated**: April 1, 2026
