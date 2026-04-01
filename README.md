# Link Mage

<div align="center">

**Graph Visualization & OSINT Analysis Tool**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.77.2+-orange.svg)](https://www.rust-lang.org/)
[![Node.js](https://img.shields.io/badge/Node.js-18+-green.svg)](https://nodejs.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app/)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Documentation](#-documentation) • [Contributing](#-contributing) • [License](#-license)

</div>

---

## 📖 Overview

**Link Mage** is a professional-grade graph visualization and OSINT (Open-Source Intelligence) analysis tool inspired by Maltego. It helps investigators, security researchers, and analysts collect, visualize, and analyze relationships between data points through an interactive graph interface.

### What It Does

- **Entity Mapping**: Represents data as nodes (people, domains, emails, IPs, organizations)
- **Relationship Discovery**: Shows connections between entities
- **Automated Enrichment**: Runs predefined queries ("Transforms") to expand datasets
- **Graph Analysis**: Visual relationship mapping with advanced filtering and statistics

### Use Cases

- 🔍 **Cybersecurity investigations** - Attack surface mapping, threat intelligence
- 🕵️ **OSINT investigations** - Tracing infrastructure, identifying relationships
- 📊 **Network analysis** - Mapping networks of activity
- 🔎 **Digital forensics** - Evidence correlation, link analysis
- ⚠️ **Fraud detection** - Identifying connected entities

---

## ✨ Features

### Core Features

#### 🔗 Graph Visualization
- Interactive node-link diagrams using Cytoscape.js
- Force-directed and hierarchical layouts
- Real-time filtering and search
- Graph statistics and analytics

#### 🎯 Entity-Based Data Modeling
- 15+ entity types (IP, Domain, Email, Person, Organization, etc.)
- Customizable visual styling (icons, colors, shapes)
- Entity registry with automatic styling

#### 🔄 Transform System
- **Built-in Transforms**: DNS lookup, WHOIS, GeoIP, Web scraping, VirusTotal
- **Remote Transforms**: HTTP-based transforms in any language
- **Native Plugins**: Rust SDK for high-performance plugins
- **Transform Pipelines**: Multi-step automated workflows

#### 📊 Data Provenance Tracking
- Automatic source tracking for every entity
- Transform chain history
- Confidence scoring with decay algorithm
- Forensic integrity controls

#### 🏷️ Annotations & Tagging
- Custom tags with 10 color options
- Rich text annotations
- Pin important findings
- Usage tracking and filtering

#### ⌨️ Power User Features
- 60+ keyboard shortcuts
- Advanced filtering (7 types)
- Quick filter presets
- Real-time graph statistics

### Technical Features

- **Standalone & Private**: Works offline, no cloud dependencies
- **Secure**: API keys stored in OS keychain
- **High Performance**: Optimized for 10,000+ node graphs
- **Cross-Platform**: macOS, Windows, Linux support
- **Extensible**: Plugin system for custom transforms

---

## 🚀 Installation

### Prerequisites

- **Node.js** 18 or higher
- **Rust** 1.77.2 or higher
- **Tauri dependencies** (see [Tauri setup guide](https://tauri.app/start/prerequisites/))

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/link-mage.git
cd link-mage

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Pre-built Binaries

Download pre-built installers from the [Releases page](https://github.com/your-username/link-mage/releases):

- **macOS**: `.dmg` (Universal: Intel + Apple Silicon)
- **Windows**: `.msi` and `.exe` installers
- **Linux**: `.deb`, `.rpm`, `.AppImage`

### System Requirements

**Minimum:**
- OS: macOS 10.15+ / Windows 10+ / Ubuntu 20.04+
- CPU: Intel Core i3 or equivalent
- RAM: 4 GB
- Storage: 500 MB free space

**Recommended:**
- OS: Latest version
- CPU: Intel Core i5 or equivalent
- RAM: 8 GB
- Storage: 1 GB free space

---

## 📖 Usage

### Quick Start

1. **Launch Link Mage**
2. **Add a node**: Sidebar → Add Entity → Select type → Enter value
3. **Run a transform**: Right-click node → Select transform
4. **View results**: New nodes and edges appear automatically

### Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|--------------|-------|
| New graph | `Ctrl+N` | `⌘N` |
| Save graph | `Ctrl+S` | `⌘S` |
| Fit graph | `Ctrl+F` | `⌘F` |
| Auto layout | `Ctrl+L` | `⌘L` |
| Select all | `Ctrl+A` | `⌘A` |
| Help | `?` | `?` |

**Full shortcut reference**: Press `?` in the application or see [QUICK_REFERENCE.md](QUICK_REFERENCE.md)

### Documentation

- **[FEATURES.md](FEATURES.md)** - Complete feature documentation
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Quick reference card
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Testing instructions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines

---

## 🔌 Extending Link Mage

### Remote Transforms

Create transforms in any language by hosting an HTTP endpoint:

**Request:**
```json
{
  "id": "node-uuid",
  "type": "Domain",
  "properties": { "value": "example.com" }
}
```

**Response:**
```json
{
  "new_nodes": [
    { "id": "new-uuid", "type": "IP", "properties": { "value": "93.184.216.34" } }
  ],
  "new_edges": [
    { "id": "edge-uuid", "source": "node-uuid", "target": "new-uuid", "label": "resolves-to" }
  ]
}
```

### Native Plugins

Build high-performance plugins in Rust using the provided SDK:

```rust
// Example plugin
use link_mage_sdk::{Node, Edge, Transform};

pub struct MyPlugin;

impl Transform for MyPlugin {
    fn run(&self, seed: &Node) -> Result<(Vec<Node>, Vec<Edge>), String> {
        // Your transform logic
        Ok((nodes, edges))
    }
}
```

See `plugins/example/` for a complete example.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Frontend (SvelteKit)                    │
│  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐  │
│  │  Graph    │  │  Sidebar  │  │   Settings Panel    │  │
│  │(Cytoscape)│  │(Controls) │  │ (API Keys, Config)  │  │
│  └───────────┘  └───────────┘  └─────────────────────┘  │
│                        │                                 │
│                 Tauri invoke()                           │
└────────────────────────┼────────────────────────────────┘
                         │ IPC
┌────────────────────────▼────────────────────────────────┐
│                   Backend (Rust/Tauri)                   │
│  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐  │
│  │ commands  │  │  graph    │  │    transforms       │  │
│  │   (IPC)   │  │  (data)   │  │  (DNS, WHOIS, VT)   │  │
│  └───────────┘  └───────────┘  └─────────────────────┘  │
│  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐  │
│  │  plugins  │  │ keychain  │  │    provenance       │  │
│  │   (SDK)   │  │ (secure)  │  │   (tracking)        │  │
│  └───────────┘  └───────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Tech Stack

- **Frontend**: [SvelteKit 5](https://svelte.dev/), [TypeScript](https://www.typescriptlang.org/)
- **Backend**: [Rust](https://www.rust-lang.org/), [Tauri v2](https://tauri.app/)
- **Visualization**: [Cytoscape.js](https://js.cytoscape.org/)
- **Testing**: Rust std::test, TypeScript type-checking

---

## 🧪 Testing

```bash
# TypeScript type checking
npm run check

# Rust tests
cd src-tauri && cargo test --lib

# Build for production
npm run build
npm run tauri build
```

**Test Coverage**: ~90%  
**Tests**: 150+ automated tests

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Fork the repository
git clone https://github.com/your-username/link-mage.git
cd link-mage

# Create a branch
git checkout -b feature/amazing-feature

# Make your changes
# ...

# Test your changes
npm run check
cd src-tauri && cargo test

# Commit and push
git commit -m "feat: add amazing feature"
git push origin feature/amazing-feature
```

### Areas We Need Help

- 🐛 **Bug fixes** - See [open issues](https://github.com/your-username/link-mage/issues)
- ✨ **New features** - Transforms, entity types, UI improvements
- 📖 **Documentation** - Tutorials, examples, translations
- 🧪 **Testing** - Write tests, improve coverage
- 🎨 **Design** - UI/UX improvements, icons, themes

### Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) to keep our community welcoming and inclusive.

---

## 🔒 Security

### Reporting Vulnerabilities

**Do not create public issues for security vulnerabilities.**

Please report security issues via:
- [GitHub Security Advisories](https://github.com/your-username/link-mage/security/advisories)
- Email: [security email]

See our [Security Policy](SECURITY.md) for details.

### Security Features

- ✅ Content Security Policy (CSP)
- ✅ CSV injection protection
- ✅ Input validation
- ✅ Secure credential storage (OS keychain)
- ✅ Error message sanitization

---

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for details.

```
Copyright (c) 2026 Link Mage Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 🙏 Acknowledgments

- Inspired by [Maltego](https://www.maltego.com/) and similar OSINT tools
- Built with [SvelteKit](https://kit.svelte.dev/), [Tauri](https://tauri.app/), and [Cytoscape.js](https://js.cytoscape.org/)
- Thanks to all contributors and the open-source community

---

## 📬 Contact

- **Website**: [Link Mage](https://github.com/your-username/link-mage)
- **Issues**: [GitHub Issues](https://github.com/your-username/link-mage/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/link-mage/discussions)
- **Twitter**: [@yourusername](https://twitter.com/yourusername) (if applicable)

---

<div align="center">

**Made with ❤️ by the Link Mage Contributors**

[⬆ Back to top](#link-mage)

</div>
