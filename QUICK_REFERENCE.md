# Link Mage - Quick Reference Card

## 🎯 Provenance Tracking
- **View**: Select node → Provenance panel
- **Confidence**: Green (80%+) | Yellow (50-79%) | Red (<50%)
- **Sources**: Manual (100%) > DNS/VT (95%) > WHOIS (90%) > GeoIP (85%) > Scrape (75%) > Remote (70%)

## 🏷️ Tags & Annotations
- **Create Tag**: Tags & Notes → Name + Color → +Tag
- **Apply Tag**: Select node → Click + on tag
- **Add Note**: Select node → Type note → Choose color → Add
- **Pin Note**: Click 📍 to pin important annotations

## ⚙️ Machines (multi-step transforms)
- **Run**: Right-click node → select machine
- **Monitor**: Check status bar for progress and per-step results

## ⌨️ Essential Shortcuts
```
File:     ⌘N New | ⌘O Open | ⌘S Save | ⌘⇧S Save As
View:     ⌘F Fit | ⌘+ Zoom In | ⌘- Zoom Out | ⌘L Layout
Select:   ⌘A All | Escape None | Delete Remove
Tools:    ? Help | ⌘⇧F Clear Filters | ⌘B Sidebar | ⌘, Settings
```

## 🔍 Filtering
- **Quick**: Click preset buttons (High Confidence, Hubs, etc.)
- **Types**: Click type chips to toggle
- **Degree**: Adjust min/max sliders
- **Search**: Type in search box (debounced 300ms)
- **Reset**: "Reset All Filters" button

## 🎨 Visual Indicators
- 🟢 Green = High confidence / Success
- 🟡 Yellow = Medium confidence / Warning  
- 🔴 Red = Low confidence / Error
- ⭐ Hub = 5+ connections
- 🔍 Isolated = No connections
- 📌 Pinned = Important annotation

## 📁 File Operations
- **New**: ⌘N | Confirm dialog
- **Save**: ⌘S | Smart save (dialog if first time)
- **Load**: ⌘O | File picker
- **Export**: CSV, GraphML from Sidebar

## 🔄 Transform Workflow
1. Add seed node (e.g., domain)
2. Right-click → Select transform
3. Wait for results
4. Check provenance
5. Tag important findings
6. Add annotations
7. Run a machine for multi-step automation

## 💡 Pro Tips
- Press `?` anytime for full shortcut reference
- Use machines for repetitive investigations
- Tag critical findings immediately
- Pin important annotations
- Filter by confidence for quality focus
- Check stats to identify key nodes
- Use degree filter to find hubs
- Search across all properties

---

**Full Documentation**: See FEATURES.md, TESTING_GUIDE.md, ENHANCEMENTS_SUMMARY.md
