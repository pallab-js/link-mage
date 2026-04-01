# Link Mage - New Features Documentation

## Table of Contents
1. [Data Provenance Tracking](#1-data-provenance-tracking)
2. [Annotations & Tagging](#2-annotations--tagging)
3. [Transform Pipelines](#3-transform-pipelines)
4. [Keyboard Shortcuts](#4-keyboard-shortcuts)
5. [Advanced Filtering](#5-advanced-filtering)
6. [Graph Statistics](#6-graph-statistics)

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

## 3. Transform Pipelines

### Overview
Automate multi-step intelligence gathering with predefined transform sequences.

### Built-in Pipelines

#### 1. Domain Enrichment
**Purpose**: Comprehensive domain intelligence
**Steps**:
1. DNS Lookup (Domain → IP)
2. GeoIP Lookup (IP → Location)
3. WHOIS Lookup (Domain → Owner info)

**Use Case**: Initial domain investigation

#### 2. IP Reconnaissance
**Purpose**: IP address intelligence
**Steps**:
1. GeoIP Lookup (IP → Location)
2. Reverse DNS (IP → Hostnames)

**Use Case**: Infrastructure analysis

#### 3. Email Investigation
**Purpose**: Email address tracing
**Steps**:
1. Domain Search (Email → Domain)
2. DNS Lookup (Domain → IP)
3. WHOIS Lookup (Domain → Owner)

**Use Case**: Email origin tracing

#### 4. Malware Analysis
**Purpose**: File hash reputation check
**Steps**:
1. VirusTotal Hash lookup

**Use Case**: Malware identification

### Creating Custom Pipelines

#### Pipeline Configuration
```typescript
interface PipelineStep {
  transformName: string;    // Transform to execute
  inputType: string;        // Required node type
  description: string;      // Step description
  timeout: number;          // Timeout in seconds
  continueOnError: boolean; // Continue if fails
  maxResults: number;       // Limit results (0 = unlimited)
}
```

#### Best Practices
1. **Start specific**: Begin with specific transforms
2. **Handle errors**: Enable continueOnError for non-critical steps
3. **Set timeouts**: Prevent hanging on slow transforms
4. **Limit results**: Use maxResults to control graph size

### Execution Flow

1. **Select seed node**
2. **Choose pipeline** from context menu
3. **Sequential execution**:
   - Step 1 runs on seed
   - Results become input for Step 2
   - Continues through all steps
4. **Results added** to graph automatically

### Monitoring

#### Execution Status
- ⏳ **Pending**: Step waiting to run
- 🔄 **Running**: Currently executing
- ✅ **Success**: Completed successfully
- ❌ **Failed**: Error occurred
- ⏭️ **Skipped**: Not executed

#### Results Tracking
- Nodes added per step
- Edges created
- Duration per step
- Total execution time
- Error messages (if any)

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
?        Show keyboard help
Mod+K    Command palette
Mod+T    Toggle tags panel
Mod+M    Add annotation
Mod+P    Run pipeline
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

## 6. Graph Statistics

### Overview
Real-time analytics providing insights into graph structure and composition.

### Metrics

#### Basic Stats
- **Total Nodes**: Count of all entities
- **Total Edges**: Count of all relationships
- **Density**: Connectivity ratio (0-100%)
- **Components**: Connected subgraphs count

#### Degree Analysis
- **Average Degree**: Mean connections per node
- **Max Degree**: Most-connected node
- **Min Degree**: Least-connected node

#### Node Classification
- **Isolated Nodes**: Degree = 0 (no connections)
- **Hub Nodes**: Degree ≥ 5 (highly connected)

#### Type Distribution
- Count per entity type
- Visual bar chart
- Percentage breakdown

### Statistics Panel

#### Layout
```
┌─────────────────────────────────┐
│ 📊 Graph Statistics         [📋]│
├─────────────────────────────────┤
│ [Nodes] [Edges] [Density] [⊕]  │
│                                 │
│ Degree Distribution             │
│   Average: 2.5                  │
│   Max: 12                       │
│   Min: 0                        │
│                                 │
│ Special Nodes                   │
│   🔍 Isolated: 3                │
│   ⭐ Hubs (5+): 2               │
│                                 │
│ Nodes by Type                   │
│   Domain    ████████ 8          │
│   IP        ██████   6          │
│   Location  ██       2          │
│                                 │
│ Top Hubs                        │
│  ⑫ example.com    Domain        │
│  ⑧  192.168.1.1    IP           │
└─────────────────────────────────┘
```

#### Features
- **Auto-update**: Refreshes when graph changes
- **Copy button**: Export stats to clipboard
- **Visual bars**: Type distribution chart
- **Hub list**: Top 5 most-connected nodes

### Metrics Explained

#### Graph Density
Measures how connected the graph is:
```
Density = 2E / (V × (V-1))
```
Where E = edges, V = nodes

- **0%**: No edges (isolated nodes)
- **100%**: Complete graph (all possible edges)

#### Connected Components
Groups of nodes where each node is reachable from any other node in the same group.

**Algorithm**: Breadth-first search (BFS)

#### Degree Centrality
Measures node importance based on connections:
```
Centrality = degree / (n - 1)
```
Where n = total nodes

### Use Cases

#### Quality Assessment
- High density → Well-connected investigation
- Many isolated nodes → Incomplete data
- Multiple components → Separate investigations

#### Focus Areas
- Hub nodes → Key entities to investigate
- Low confidence → Verify data sources
- Recent activity → Active investigation areas

---

## Appendix

### File Locations

```
src/lib/
  types/
    provenance.ts      # Provenance types
    pipeline.ts        # Pipeline types
  stores/
    annotations.ts     # Annotation/tag stores
  utils/
    graphStats.ts      # Statistics calculations
  components/
    ProvenancePanel.svelte
    TagManager.svelte
    ShortcutHelp.svelte
    StatsPanel.svelte
    FilterPanel.svelte

src-tauri/src/
  provenance.rs        # Rust provenance module
  pipeline.rs          # Rust pipeline engine
```

### Configuration

#### Confidence Decay
Edit in `provenance.ts`:
```typescript
const decayFactor = 0.98;  // 2% per step
```

#### Hub Threshold
Edit in `graphStats.ts`:
```typescript
const HUB_THRESHOLD = 5;  // Connections for hub status
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

#### Pipelines Not Running
- Verify transforms are available
- Check seed node type matches pipeline input
- Review error messages in status bar

#### Filters Not Working
- Check if invert mode is enabled
- Verify filter criteria is set
- Try resetting all filters

---

**Version**: 0.1.0 Enhanced  
**Last Updated**: April 1, 2026
