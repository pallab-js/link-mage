# Link Mage Project Context

## Project Overview
**Link Mage** is a graph visualization and analysis application built with **Tauri v2** and **SvelteKit 5**. It allows users to create, manage, and analyze complex networks of data (nodes and edges) with a focus on OSINT and infrastructure analysis.

### Core Architecture
- **Backend (Rust/Tauri):** Manages the application state, file I/O (CSV, GraphML), secure API key storage (using system keychain via `keyring`), and a dynamic plugin system.
- **Frontend (SvelteKit/Cytoscape):** Provides a reactive UI for graph manipulation, using `cytoscape.js` for high-performance graph rendering and layout.
- **Plugin System:** A modular architecture where "transforms" (data enrichment tasks like DNS lookups or WHOIS queries) can be implemented as built-in Rust modules or dynamically loaded `.dylib`/`.so`/`.dll` plugins using the provided SDK.

## Tech Stack
- **Framework:** [Tauri v2](https://tauri.app/) (Rust backend, Webview frontend)
- **Frontend:** [SvelteKit 5](https://svelte.dev/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/)
- **Visualization:** [Cytoscape.js](https://js.cytoscape.org/)
- **Backend Dependencies:**
  - `libloading`: For dynamic plugin loading.
  - `keyring`: For secure API key management.
  - `hickory-resolver`: For DNS transforms.
  - `whois-rust`: For WHOIS transforms.
  - `maxminddb`: For GeoIP transforms.
- **Styling:** CSS (likely Svelte-native or global).

## Project Structure
- `src-tauri/`: Rust backend source code.
  - `src/commands.rs`: Tauri command definitions for frontend-to-backend communication.
  - `src/plugin_api.rs`: Logic for scanning and loading external plugins.
  - `src/transforms.rs`: Built-in data enrichment logic.
- `src/`: SvelteKit frontend source code.
  - `src/lib/components/Graph.svelte`: The primary graph visualization component.
  - `src/lib/stores.ts`: Global state management for the graph and application settings.
- `plugins/`:
  - `sdk/`: Rust crate providing the trait and macros for building Link Mage plugins.
  - `example/`: A sample plugin implementation (e.g., Reverse DNS).

## Building and Running

### Prerequisites
- Node.js (v18+)
- Rust (v1.77.2+)
- Tauri CLI (`cargo install tauri-cli` or use `npm run tauri`)

### Development
```sh
# Install dependencies
npm install

# Start the development server (runs SvelteKit and Tauri)
npm run dev # or npm run tauri dev
```

### Building
```sh
# Build for production
npm run build # or npm run tauri build
```

### Plugins
To build a plugin:
```sh
cd plugins/example
cargo build --release
```
The resulting library (e.g., `liblm_plugin_reverse_dns.dylib`) should be placed in the application's `plugins` data directory to be auto-loaded.

## Development Conventions
- **Svelte 5:** Uses snippets and runes (e.g., `$state`, `$derived`).
- **Tauri Commands:** New features requiring backend access should be implemented as Tauri commands in `src-tauri/src/commands.rs` and registered in `lib.rs`.
- **State Management:** The frontend uses Svelte stores (`src/lib/stores.ts`) to sync with the Rust backend's `AppState`.
- **Transforms:** All data enrichment should follow the `Transform` trait pattern defined in the plugin SDK to ensure compatibility between built-in and external modules.
