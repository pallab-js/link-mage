# Link Mage

**A desktop OSINT link-analysis tool** — build interactive entity graphs, enrich them
with one-click transforms, and keep full provenance on every fact.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-5-green.svg)](https://svelte.dev/)

Inspired by Maltego. Runs fully offline on macOS, Windows, and Linux — your data and
API keys never leave your machine (keys are stored in the OS keychain).

## Features

- **Entity graph** — 15 node types (IP, Domain, Email, Person, Hash, …) on a Cytoscape.js canvas with multiple layouts
- **Transforms** — enrich nodes with DNS, WHOIS, GeoIP, web-scraping, and VirusTotal lookups
- **Machines** — chain transforms into multi-step runs (Domain Footprint, IP Intel)
- **Provenance** — every node records its source, transform chain, and a decaying confidence score
- **Extensible** — remote transforms (any HTTP endpoint) and native Rust plugins
- **Tags & notes** — colour tags and annotations on nodes and edges
- **Filtering** — type, degree, search, provenance source, confidence, date range, invert mode
- **Private by default** — no cloud, no telemetry

## Quick Start

### Prerequisites

[Node.js 20+](https://nodejs.org/), [Rust](https://www.rust-lang.org/), and the
[Tauri v2 platform dependencies](https://tauri.app/start/prerequisites/).

### Run from source

```bash
git clone https://github.com/pallab-js/link-mage.git
cd link-mage
npm install
npm run tauri dev
```

## Usage

1. Add a seed node from the sidebar (e.g. type `Domain`, value `example.com`)
2. Right-click it and run a transform or machine
3. Results appear as new nodes with provenance attached
4. Filter, tag, and annotate as the graph grows; save with `⌘S` / `Ctrl+S`

Press `?` in the app for the full keyboard-shortcut reference.

## Extending

### Remote transforms

Host an HTTP endpoint in any language. Link Mage POSTs the seed node:

```json
{ "id": "node-uuid", "type": "Domain", "properties": { "value": "example.com" } }
```

and expects:

```json
{
  "new_nodes": [{ "id": "new-uuid", "type": "IP", "properties": { "value": "93.184.216.34" } }],
  "new_edges": [{ "id": "edge-uuid", "source": "node-uuid", "target": "new-uuid", "label": "resolves-to" }]
}
```

### Native plugins

Build high-performance plugins in Rust with the SDK in [`plugins/sdk`](plugins/sdk) —
see [`plugins/example`](plugins/example) for a complete working plugin.

## Development

```bash
npm run check                 # TypeScript / Svelte type check
npm test                      # Frontend tests (Vitest)
cd src-tauri && cargo test    # Backend tests
npm run tauri build           # Production build
```

## Documentation

- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) — one-page cheat sheet
- [FEATURES.md](FEATURES.md) — feature deep-dives
- [TESTING_GUIDE.md](TESTING_GUIDE.md) — manual test checklist
- [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) — full documentation index

## Contributing

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Report security issues privately via
[GitHub Security Advisories](https://github.com/pallab-js/link-mage/security/advisories)
(see [SECURITY.md](SECURITY.md)).

## License

MIT — see [LICENSE](LICENSE).
