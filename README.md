# Link Mage

A desktop OSINT link-analysis tool. Build interactive entity graphs, enrich them with
one-click transforms, and keep full provenance on every fact.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-ff3e00.svg)](https://svelte.dev/)

Inspired by Maltego. Runs fully offline on macOS, Windows, and Linux — your data and
API keys never leave your machine.

## Features

- **Entity graph** — 15 node types (IP, Domain, Email, Person, Hash, …) on a
  Cytoscape.js canvas with multiple layouts.
- **Transforms** — enrich nodes via DNS, WHOIS, GeoIP, web scraping, and VirusTotal.
- **Machines** — chain transforms into multi-step runs (Domain Footprint, IP Intel).
- **Provenance** — every node records its source, transform chain, and a confidence score.
- **Extensible** — remote HTTP transforms and native Rust plugins.
- **Private by default** — no cloud, no telemetry; API keys live in the OS keychain.

## Quick start

**Prerequisites:** [Node.js 20+](https://nodejs.org/), [Rust](https://www.rust-lang.org/),
and the [Tauri v2 platform dependencies](https://tauri.app/start/prerequisites/).

```bash
git clone https://github.com/pallab-js/link-mage.git
cd link-mage
npm install
npm run tauri dev
```

## Usage

1. Add a seed node from the sidebar (e.g. type `Domain`, value `example.com`).
2. Right-click it and run a transform or machine.
3. Results appear as new nodes with provenance attached.
4. Filter, tag, and annotate as the graph grows; save with `⌘S` / `Ctrl+S`.

Press `?` in the app for the full keyboard-shortcut reference.

## Extending

### Remote transforms

Host an HTTP endpoint in any language. Link Mage POSTs the seed node and expects new
nodes and edges in return:

```jsonc
// Request body
{ "id": "node-uuid", "type": "Domain", "properties": { "value": "example.com" } }

// Expected response
{
  "new_nodes": [{ "id": "new-uuid", "type": "IP", "properties": { "value": "93.184.216.34" } }],
  "new_edges": [{ "id": "edge-uuid", "source": "node-uuid", "target": "new-uuid", "label": "resolves-to" }]
}
```

Only `http`/`https` endpoints that resolve to public addresses are allowed; requests to
loopback, private, and link-local ranges are blocked.

### Native plugins

Build Rust plugins with the SDK in [`plugins/sdk`](plugins/sdk); see
[`plugins/example`](plugins/example) for a working reference. Native plugins run
in-process, so auto-loading is opt-in via an `ENABLE_PLUGINS` marker file in the plugin
directory.

## Development

```bash
npm run check              # TypeScript / Svelte type check
npm test                   # Frontend tests (Vitest)
npm run tauri build        # Production build
cd src-tauri && cargo test # Backend tests
```

## Documentation

| Document | Purpose |
| --- | --- |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | One-page cheat sheet |
| [FEATURES.md](FEATURES.md) | Feature deep-dives |
| [TESTING_GUIDE.md](TESTING_GUIDE.md) | Manual test checklist |
| [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) | Full documentation index |

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Report security issues privately via
[GitHub Security Advisories](https://github.com/pallab-js/link-mage/security/advisories)
(see [SECURITY.md](SECURITY.md)).

## License

[MIT](LICENSE)
