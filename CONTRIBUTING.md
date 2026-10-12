# Contributing to Link Mage

Thank you for your interest in contributing to Link Mage! This document provides guidelines and instructions for contributing.

## 🎯 Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) to keep our community welcoming and inclusive.

## 📋 How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the behavior
- **Expected vs actual behavior**
- **Screenshots** if applicable
- **Environment details** (OS, version, etc.)

**Example:**
```markdown
**Bug:** Application crashes when running DNS Lookup transform

**Steps to Reproduce:**
1. Create domain node with value "example.com"
2. Right-click → DNS Lookup
3. Application crashes

**Expected:** IP nodes should be created
**Actual:** Application crashes with error message

**Environment:** macOS 14.0, Link Mage v0.1.0
```

### Suggesting Features

Feature suggestions are welcome! Please provide:

- **Use case** - Why is this feature needed?
- **Proposed solution** - How should it work?
- **Alternatives considered** - Other approaches?
- **Additional context** - Screenshots, mockups, etc.

### Pull Requests

1. **Fork** the repository
2. **Create a branch** from `main`
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
4. **Test thoroughly**
5. **Commit** with clear messages
   ```bash
   git commit -m "feat: add amazing feature"
   ```
6. **Push** to your fork
   ```bash
   git push origin feature/amazing-feature
   ```
7. **Open a Pull Request**

### Pull Request Guidelines

- **Fill in the template** - Complete the PR template
- **Reference issues** - Link to related issues
- **Describe changes** - What did you change and why?
- **Include tests** - Add tests for new features
- **Update documentation** - Keep docs in sync
- **Keep it focused** - One feature/fix per PR

## 🏗️ Development Setup

### Prerequisites

- **Node.js** 18+ 
- **Rust** 1.77.2+
- **npm** or **yarn**

### Installation

```bash
# Clone the repository
git clone https://github.com/your-username/link-mage.git
cd link-mage

# Install dependencies
npm install

# Start development
npm run tauri dev
```

### Running Tests

```bash
# TypeScript type checking
npm run check

# Rust tests
cd src-tauri
cargo test --lib

# Frontend tests
npm test

# Build for production
npm run build
npm run tauri build
```

## 📐 Coding Standards

### TypeScript/Svelte

- Use **TypeScript** for type safety
- Follow **Svelte 5** conventions
- Use **functional components** with `$state`
- Add **JSDoc comments** for public APIs
- Keep components **focused and modular**

```typescript
/**
 * Calculate graph statistics
 * @param graph - The graph to analyze
 * @returns Statistics object
 */
export function calculateStats(graph: Graph): GraphStats {
  // Implementation
}
```

### Rust

- Follow **Rust idioms** and best practices
- Use **descriptive variable names**
- Add **documentation comments** (`///`)
- Handle **errors properly** (Result, Option)
- Write **unit tests** for modules

```rust
/// Calculate confidence score based on source and transform chain
/// 
/// # Arguments
/// * `source` - The provenance source type
/// * `chain` - The transform chain
/// 
/// # Returns
/// Confidence score between 0.0 and 1.0
pub fn calculate_confidence(source: &ProvenanceSource, chain: &[TransformStep]) -> f64 {
    // Implementation
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting)
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Build/config changes

**Examples:**
```bash
feat: add machine run tracking
fix: resolve memory leak in status timer
docs: update FEATURES.md with new shortcuts
refactor: simplify filter store logic
```

## 🧪 Testing Guidelines

### Writing Tests

- **Unit tests** for utility functions
- **Integration tests** for workflows
- **Test edge cases** and error conditions
- **Aim for >80% coverage**

### Running Tests

```bash
# Before committing
npm run check          # TypeScript
npm test               # Frontend (Vitest)
cd src-tauri && cargo test  # Rust
```

## 📖 Documentation

### Documentation Files

- **README.md** - Project overview
- **FEATURES.md** - Feature documentation
- **QUICK_REFERENCE.md** - Quick start guide
- **CONTRIBUTING.md** - This file
- **CODE_OF_CONDUCT.md** - Community guidelines

### Updating Documentation

- Keep documentation **up-to-date** with code changes
- Use **clear, concise language**
- Include **examples** where helpful
- Add **screenshots** for UI changes

## 🔒 Security

### Reporting Security Issues

**Do not create public issues for security vulnerabilities.**

Email security concerns to: [security email or use GitHub Security Advisories]

### Security Best Practices

- Never commit **API keys** or **secrets**
- Validate **all user input**
- Use **parameterized queries** (if applicable)
- Keep **dependencies updated**

## 📝 License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).

## ❓ Questions?

- **Check existing issues** - Your question may already be answered
- **Create a discussion** - For general questions
- **Contact maintainers** - For private inquiries

---

Thank you for contributing to Link Mage! 🎉
