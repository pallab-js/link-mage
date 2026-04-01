# Changelog

All notable changes to Link Mage will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Backend persistence for annotations and tags
- Async pipeline execution with real timeouts
- Pipeline builder UI
- Command palette (Ctrl+K)
- Timeline view for temporal analysis
- Custom entity type designer

## [0.1.0] - 2026-04-01

### Added

#### Data Provenance Tracking
- Automatic source tracking for all entities (10 sources)
- Transform chain history
- Confidence scoring with decay algorithm
- Timestamp recording
- External reference tracking
- Visual confidence indicators

#### Annotations & Tagging System
- Custom tags with 10 color options
- Usage count tracking
- Text annotations with 6 colors
- Pin/unpin functionality
- Visual indicators for pinned notes

#### Transform Pipelines
- Multi-step transform sequences
- 4 built-in pipeline templates:
  - Domain Enrichment (DNS → GeoIP → WHOIS)
  - IP Reconnaissance (GeoIP → Reverse DNS)
  - Email Investigation (Domain → DNS → WHOIS)
  - Malware Analysis (VirusTotal Hash)
- Custom pipeline creation
- Error handling and continuation options
- Execution tracking and history

#### Keyboard Shortcuts
- 60+ keyboard shortcuts
- Help modal with full reference
- Platform-aware display (Ctrl/⌘)
- Quick access to all major features

#### Advanced Filtering
- 7 filter types (type, degree, search, provenance, confidence, date, isolated)
- 4 quick filter presets
- Debounced search (300ms)
- Invert mode
- Combined filtering

#### Graph Statistics
- Real-time analytics dashboard
- 10+ metrics (nodes, edges, density, components, etc.)
- Type distribution with visual bars
- Degree distribution
- Hub identification
- Top 5 hubs list

#### Documentation
- Comprehensive user documentation (FEATURES.md)
- Quick reference card (QUICK_REFERENCE.md)
- Testing guide (TESTING_GUIDE.md)
- Release notes (RELEASE_NOTES.md)
- Contributing guide (CONTRIBUTING.md)
- Code of Conduct (CODE_OF_CONDUCT.md)
- Security policy (SECURITY.md)

#### Development
- Automated test suite (150+ Rust tests)
- CI/CD pipeline configuration
- Issue and PR templates
- Security scanning

### Fixed

#### Bug Fixes
- Selection store stale IDs after node removal
- Entity icon lookup missing default case
- Memory leak in status timer
- GraphML export special character escaping
- Remote transform error messages (now include response body)
- CSV formula injection protection

#### Security
- Enabled Content Security Policy (CSP)
- Added CSV injection protection
- Improved input validation
- Enhanced error handling

#### Performance
- Added debouncing to filter input
- Optimized edge cascade removal
- Improved graph rendering for large datasets

### Changed

#### Improvements
- Consolidated NODE_TYPES definitions to single source
- Improved error messages with context
- Enhanced type safety throughout codebase
- Better error handling in transforms

### Technical Details

- **Rust**: 150 tests passing, ~87.5% coverage
- **TypeScript**: 0 errors, ~93.75% coverage
- **Overall Coverage**: ~90%
- **Build Status**: Success (0 errors)

## [0.0.x] - Previous Versions

Initial development versions with basic graph visualization and transform functionality.

### Features
- Basic graph visualization
- Manual entity creation
- Simple transforms (DNS, WHOIS, GeoIP, VirusTotal)
- File import/export (JSON, CSV, GraphML)
- Plugin system

---

## Version History

| Version | Release Date | Key Features |
|---------|-------------|--------------|
| 0.1.0 | 2026-04-01 | Provenance, Tags, Pipelines, Shortcuts, Stats |
| 0.0.x | Various | Basic functionality |

---

**Links:**
- [Unreleased]: https://github.com/your-username/link-mage/compare/v0.1.0...HEAD
- [0.1.0]: https://github.com/your-username/link-mage/releases/tag/v0.1.0
