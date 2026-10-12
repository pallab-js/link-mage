# Link Mage - Documentation Index

## Current Documentation

### Getting Started
- **[README.md](README.md)** - Overview, install, build, and test instructions
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - One-page cheat sheet (provenance, tags, machines, shortcuts, filters)

### Features & Usage
- **[FEATURES.md](FEATURES.md)** - Feature documentation
  1. Data Provenance Tracking
  2. Annotations & Tagging
  3. Transform Machines
  4. Keyboard Shortcuts
  5. Advanced Filtering
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Manual testing checklist (machines, shortcuts, filtering, provenance)

### Development
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution workflow, commit conventions, test commands
- **[SECURITY.md](SECURITY.md)** - Security policy and reporting
- **[CHANGELOG.md](CHANGELOG.md)** - Release history
- **[RELEASE_NOTES.md](RELEASE_NOTES.md)** - Release notes
- **[GITHUB_SETUP_GUIDE.md](GITHUB_SETUP_GUIDE.md)** - GitHub configuration guide
- **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** - Community standards

### Automated Tests
```bash
cd src-tauri && cargo test   # ~130 Rust tests
npm test                     # Vitest frontend tests
```

## Historical Records (docs/)
Snapshot documents from earlier development phases. They may mention
features that have since changed or been removed (for example the earlier
pipeline system, which was replaced by machines):
- **docs/ANALYSIS_REPORT.md** - Phase-A codebase analysis
- **docs/ENHANCEMENTS_SUMMARY.md** - Enhancement-phase implementation notes
- **docs/UAT_CHECKLIST.md** - User acceptance testing checklist (phase snapshot)
- **docs/DEPLOYMENT_CHECKLIST.md** - Deployment guide

## Source Layout
```
src/lib/
├── types/provenance.ts        # Provenance types
├── stores/                    # Graph, selection, theme, annotation stores
├── filterStore.ts             # Filtering logic (covered by Vitest)
├── components/                # Svelte UI
└── __tests__/                 # Vitest tests

src-tauri/src/
├── commands.rs                # Tauri commands
├── graph.rs                   # Graph model, JSON/CSV/GraphML I/O
├── transforms.rs              # Transforms & machines
├── provenance.rs              # Provenance & confidence
├── plugin_api.rs              # Plugin SDK boundary
└── tests/                     # Rust test modules
```
