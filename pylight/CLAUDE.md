# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-performance Python workspace symbol search language server implementation consisting of:
- **pylight**: Rust-based LSP server focusing on fast symbol search using tree-sitter
- **pydance**: TypeScript VSCode extension that wraps pylight

## Essential Commands

### Rust Development (pylight)
```bash
# Build the LSP server
cargo build --release --bin symbol_search_lsp

# Run tests
cargo test

# Format code (required before committing)
cargo fmt

# Run linter (required before committing)
cargo clippy

# Important: Always run these three commands after making changes
cargo fmt && cargo clippy && cargo test
```

### TypeScript Development (pydance)
```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch mode for development
npm run watch

# Run tests (mock tests only)
npm test

# Run integration tests (requires pylight binary)
npm run test:integration

# Lint code (required before committing)
npm run lint

# Format code
npm run format
```

### CI/CD Workflow
The project uses GitHub Actions for CI/CD. Key jobs include:
- Separate test jobs for pylight and pydance
- Multi-platform VSIX builds (Linux x64, macOS ARM64)
- Automated formatting and linting checks

## Architecture

### Rust LSP Server (pylight)
- Uses tree-sitter for Python parsing
- Implements parallel file processing with rayon
- Supports fuzzy matching for symbol search
- Binary serialization of symbol data for performance
- Main entry point: `src/symbol_search_lsp.rs`

### TypeScript Extension (pydance)
- Activates on Python workspaces
- Communicates with pylight server via LSP
- Provides workspace symbol search functionality
- Main entry point: `src/extension.ts`

### Testing Strategy
- **Rust**: In-file tests using `#[test]` attributes
- **TypeScript**: Mocha-based tests with both mock and integration tests
- Integration tests require pylight binary in extension root directory

## Key Files and Modules
- `pylight/src/symbol_search_lsp.rs`: Main LSP server implementation
- `pylight/src/symbols.rs`: Symbol extraction and processing
- `pylight/src/python.rs`: Python-specific parsing logic
- `pydance/src/extension.ts`: VSCode extension entry point
- `pydance/src/test/suite/`: Test suites including mock and integration tests